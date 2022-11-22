use cosmwasm_std::{from_binary, wasm_execute, IbcMsg, IbcTimeout, Reply, Timestamp};
use cosmwasm_std::{Addr, Coin, DepsMut, Response, SubMsg, SubMsgResponse, SubMsgResult};
use swaprouter::msg::{ExecuteMsg as SwapRouterExecute, Slipage, SwapResponse};

use crate::consts::{PACKET_LIFETIME, SWAP_REPLY_ID};
use crate::msg::ReturnTo;
use crate::state::{ForwardTo, SwapMsgReplyState, CONFIG, SWAP_REPLY_STATES};
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
        SWAP_REPLY_ID,
        &SwapMsgReplyState {
            swap_msg,
            block_time,
            forward_to: ForwardTo { channel, receiver },
        },
    )?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, SWAP_REPLY_ID)))
}

pub fn handle_swap_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let swap_msg_state = SWAP_REPLY_STATES.load(deps.storage, msg.id)?;
    SWAP_REPLY_STATES.remove(deps.storage, msg.id);

    if let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result {
        let parsed = cw_utils::parse_execute_response_data(&b)
            .map_err(|e| ContractError::CustomError { val: e.to_string() })?;
        let response: SwapResponse = from_binary(&parsed.data.unwrap())?;

        let ibc_transfer = IbcMsg::Transfer {
            channel_id: swap_msg_state.forward_to.channel,
            to_address: swap_msg_state.forward_to.receiver.into(),
            amount: Coin::new(response.amount.into(), response.send_denom),
            timeout: IbcTimeout::with_timestamp(
                swap_msg_state.block_time.plus_seconds(PACKET_LIFETIME),
            ),
        };

        // TODO: Deal with bad acks, timeouts, etc

        return Ok(Response::new().add_message(ibc_transfer));
    };

    Err(ContractError::CustomError {
        val: format!("Failed Swap"),
    })
}
