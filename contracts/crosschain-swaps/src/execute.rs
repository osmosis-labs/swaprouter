use cosmwasm_std::{
    from_binary, from_slice, to_binary, wasm_execute, IbcMsg, IbcTimeout, Reply, Timestamp,
};
use cosmwasm_std::{Addr, Coin, DepsMut, Response, SubMsg, SubMsgResponse, SubMsgResult};
use swaprouter::msg::{ExecuteMsg as SwapRouterExecute, Slipage, SwapResponse};

use crate::consts::SWAP_REPLY_ID;
use crate::msg::ReturnTo;
use crate::state::CONFIG;
use crate::ContractError;

pub fn swap_and_forward(
    deps: DepsMut,
    input_coin: Coin,
    output_denom: String,
    slipage: Slipage,
    _receiver: Addr,
    _channel: String,
    _failed_delivery: Option<ReturnTo>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let swap_msg = SwapRouterExecute::Swap {
        input_coin: input_coin.clone(),
        output_denom,
        slipage,
    };
    let msg = wasm_execute(config.swap_contract, &swap_msg, vec![input_coin])?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, SWAP_REPLY_ID)))
}

pub fn handle_swap_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    if let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result {
        let parsed = cw_utils::parse_execute_response_data(&b)
            .map_err(|e| ContractError::CustomError { val: e.to_string() })?;
        let response: SwapResponse = from_binary(&parsed.data.unwrap())?;
        deps.api.debug(&format!("inside: {response:?}"));

        let ibc_transfer = IbcMsg::Transfer {
            channel_id: "channel-123".to_string(),
            to_address: "my-special-addr".into(),
            amount: Coin::new(12345678, "uatom"),
            timeout: IbcTimeout::with_timestamp(Timestamp::from_nanos(1234567890)),
        };

        return Ok(Response::new().add_message(ibc_transfer));
    };

    Err(ContractError::CustomError {
        val: format!("Failed Swap"),
    })
}
