use cosmwasm_std::{
    coins, from_binary, to_binary, wasm_execute, BankMsg, IbcMsg, IbcTimeout, Reply, Timestamp,
};
use cosmwasm_std::{Addr, Coin, DepsMut, Response, SubMsg, SubMsgResponse, SubMsgResult};
use swaprouter::msg::{ExecuteMsg as SwapRouterExecute, Slipage, SwapResponse};

use crate::consts::{FORWARD_REPLY_ID, PACKET_LIFETIME, SWAP_REPLY_ID};
use crate::ibc::{MsgTransfer, MsgTransferResponse};
use crate::msg::{CrosschainSwapResponse, EventType, ListenersMsg, Recovery};

use crate::state::{
    ForwardMsgReplyState, ForwardTo, RecoveryState, Status, SwapMsgReplyState, CONFIG,
    FORWARD_REPLY_STATES, INFLIGHT_PACKETS, RECOVERY_STATES, SWAP_REPLY_STATES,
};
use crate::ContractError;

pub fn swap_and_forward(
    deps: DepsMut,
    block_time: Timestamp,
    contract_addr: Addr,
    input_coin: Coin,
    output_denom: String,
    slipage: Slipage,
    receiver: Addr,
    channel: String,
    failed_delivery: Option<Recovery>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let swap_msg = SwapRouterExecute::Swap {
        input_coin: input_coin.clone(),
        output_denom,
        slipage,
    };
    let msg = wasm_execute(config.swap_contract, &swap_msg, vec![input_coin])?;

    SWAP_REPLY_STATES.save(
        deps.storage,
        &SwapMsgReplyState {
            swap_msg,
            block_time,
            contract_addr,
            forward_to: ForwardTo {
                channel,
                receiver,
                failed_delivery,
            },
        },
    )?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, SWAP_REPLY_ID)))
}

pub fn handle_swap_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    deps.api.debug("handle_swap_reply");
    let swap_msg_state = SWAP_REPLY_STATES.load(deps.storage)?;
    SWAP_REPLY_STATES.remove(deps.storage);

    let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result else {
        return Err(ContractError::CustomError {
            val: format!("Failed Swap"),
        })
    };

    let parsed = cw_utils::parse_execute_response_data(&b)
        .map_err(|e| ContractError::CustomError { val: e.to_string() })?;
    let response: SwapResponse = from_binary(&parsed.data.unwrap())?;

    // let ibc_transfer = IbcMsg::Transfer {
    //     channel_id: swap_msg_state.forward_to.channel.clone(),
    //     to_address: swap_msg_state.forward_to.receiver.clone().into(),
    //     amount: Coin::new(
    //         response.amount.clone().into(),
    //         response.token_out_denom.clone(),
    //     ),
    //     timeout: IbcTimeout::with_timestamp(
    //         swap_msg_state.block_time.plus_seconds(PACKET_LIFETIME),
    //     ),
    // };
    let contract_addr = &swap_msg_state.contract_addr;
    let ts = swap_msg_state.block_time.plus_seconds(PACKET_LIFETIME);
    let ibc_transfer = MsgTransfer {
        source_port: "transfer".to_string(),
        source_channel: swap_msg_state.forward_to.channel.clone(),
        token: Some(
            Coin::new(
                response.amount.clone().into(),
                response.token_out_denom.clone(),
            )
            .into(),
        ),
        sender: contract_addr.to_string(),
        receiver: swap_msg_state.forward_to.receiver.clone().into(),
        timeout_height: None,
        timeout_timestamp: Some(ts.nanos()),
        memo: format!(r#"{{"callback": "{contract_addr}"}}"#),
    };

    FORWARD_REPLY_STATES.save(
        deps.storage,
        &ForwardMsgReplyState {
            channel_id: swap_msg_state.forward_to.channel,
            to_address: swap_msg_state.forward_to.receiver.into(),
            amount: response.amount.into(),
            denom: response.token_out_denom,
            failed_delivery: swap_msg_state.forward_to.failed_delivery,
        },
    )?;

    // TODO: Deal with bad acks, timeouts, etc

    Ok(Response::new()
        .add_attribute("status", "ibc_message_created")
        .add_attribute("ibc_message", format!("{:?}", ibc_transfer))
        .add_submessage(SubMsg::reply_on_success(ibc_transfer, FORWARD_REPLY_ID)))
}

use ::prost::Message; // Proveides ::decode() for MsgTransferResponse

pub fn handle_forward_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    deps.api.debug("handle_forward_reply");
    deps.api.debug(&format!("received {msg:?}"));
    let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result else {
        return Err(ContractError::CustomError { val: "invalid reply".to_string() })
    };

    let response =
        MsgTransferResponse::decode(&b[..]).map_err(|_e| ContractError::CustomError {
            val: "could not decode response".to_string(),
        })?;
    deps.api.debug(&format!("response: {response:?}"));

    let ForwardMsgReplyState {
        channel_id,
        to_address,
        amount,
        denom,
        failed_delivery,
    } = FORWARD_REPLY_STATES.load(deps.storage)?;
    FORWARD_REPLY_STATES.remove(deps.storage);

    if let Some(Recovery { recovery_addr }) = failed_delivery {
        let recovery = RecoveryState {
            recovery_addr: recovery_addr.clone(),
            channel_id: channel_id.clone(),
            sequence: response.sequence,
            amount,
            denom: denom.clone(),
            status: Status::Sent,
        };

        // Save as in-flight to be able to manipulate when the ack/timeout is received
        INFLIGHT_PACKETS.save(
            deps.storage,
            (&channel_id.clone(), response.sequence),
            &recovery,
        )?;
    };

    let response = CrosschainSwapResponse {
        msg: format!("Sent {amount}{denom} to {channel_id}/{to_address}"),
    };

    Ok(Response::new()
        .set_data(to_binary(&response)?)
        .add_attribute("status", "ibc_message_created")
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom)
        .add_attribute("channel", channel_id)
        .add_attribute("receiver", to_address))
}

pub fn recover(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let recoveries = RECOVERY_STATES.load(deps.storage, &sender)?;
    let msgs = recoveries.into_iter().map(|r| BankMsg::Send {
        to_address: r.recovery_addr.into(),
        amount: coins(r.amount, r.denom),
    });
    Ok(Response::new().add_messages(msgs))
}
