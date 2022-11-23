use cosmwasm_std::{from_binary, to_binary, wasm_execute, IbcMsg, IbcTimeout, Reply, Timestamp};
use cosmwasm_std::{Addr, Coin, DepsMut, Response, SubMsg, SubMsgResponse, SubMsgResult};
use swaprouter::msg::{ExecuteMsg as SwapRouterExecute, Slipage, SwapResponse};

use crate::consts::{FORWARD_REPLY_ID, PACKET_LIFETIME, SWAP_REPLY_ID};
use crate::msg::{CrosschainSwapResponse, ReturnTo};
use crate::state::{
    ForwardMsgReplyState, ForwardTo, SwapMsgReplyState, CONFIG, FORWARD_REPLY_STATES,
    SWAP_REPLY_STATES,
};
use crate::ContractError;

pub fn swap_and_forward(
    deps: DepsMut,
    block_time: Timestamp,
    input_coin: Coin,
    output_denom: String,
    slipage: Slipage,
    receiver: Addr,
    channel: String,
    _failed_delivery: Option<ReturnTo>,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(receiver.as_str())?;

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
            forward_to: ForwardTo { channel, receiver },
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

    let ibc_transfer = IbcMsg::Transfer {
        channel_id: swap_msg_state.forward_to.channel.clone(),
        to_address: swap_msg_state.forward_to.receiver.clone().into(),
        amount: Coin::new(
            response.amount.clone().into(),
            response.token_out_denom.clone(),
        ),
        timeout: IbcTimeout::with_timestamp(
            swap_msg_state.block_time.plus_seconds(PACKET_LIFETIME),
        ),
    };

    FORWARD_REPLY_STATES.save(
        deps.storage,
        &ForwardMsgReplyState {
            channel_id: swap_msg_state.forward_to.channel,
            to_address: swap_msg_state.forward_to.receiver.into(),
            amount: response.amount.into(),
            denom: response.token_out_denom,
        },
    );

    // TODO: Deal with bad acks, timeouts, etc

    Ok(Response::new()
        .add_attribute("status", "ibc_message_created")
        .add_attribute("ibc_message", format!("{:?}", ibc_transfer))
        .add_submessage(SubMsg::reply_on_success(ibc_transfer, FORWARD_REPLY_ID)))
}

pub fn handle_forward_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    deps.api.debug("handle_forward_reply");
    let ForwardMsgReplyState {
        channel_id,
        to_address,
        amount,
        denom,
    } = FORWARD_REPLY_STATES.load(deps.storage)?;
    FORWARD_REPLY_STATES.remove(deps.storage);

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
