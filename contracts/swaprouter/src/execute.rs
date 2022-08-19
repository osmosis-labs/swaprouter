use std::convert::TryInto;
use std::str::FromStr;

use cosmwasm_std::{
    coins, has_coins, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Reply, Response, SubMsg,
    SubMsgResponse, SubMsgResult, Uint128,
};
use osmo_bindings::OsmosisQuery;
use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountInResponse, SwapAmountInRoute};

use crate::contract::SWAP_REPLY_ID;
use crate::error::ContractError;
use crate::helpers::{self, check_is_contract_owner, generate_swap_msg, validate_pool_route};
use crate::state::{SwapMsgReplyState, ROUTING_TABLE, SWAP_REPLY_STATES};

pub fn set_route(
    deps: DepsMut,
    info: MessageInfo,
    input_denom: String,
    output_denom: String,
    pool_route: Vec<SwapAmountInRoute>,
) -> Result<Response, ContractError> {
    // only owner
    check_is_contract_owner(deps.as_ref(), info.sender)?;

    validate_pool_route(
        deps.as_ref(),
        input_denom.clone(),
        output_denom.clone(),
        pool_route.clone(),
    )?;

    ROUTING_TABLE.save(deps.storage, (&input_denom, &output_denom), &pool_route)?;

    Ok(Response::new().add_attribute("action", "set_route"))

    // TODO: add more attributes
}

pub fn trade_with_twap_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    input_token: Coin,
    output_denom: String,
    minimum_off_twap: Decimal,
) -> Result<Response, ContractError> {
    let from_time = env.block.time.minus_seconds(300);

    let depsref = deps.as_ref();

    helpers::get_multihop_twap(
        depsref::<OsmosisQuery>,
        input_token.denom,
        output_denom,
        from_time,
    );
}

pub fn trade_with_slippage_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    input_token: Coin,
    min_output_token: Coin,
) -> Result<Response, ContractError> {
    if !has_coins(&info.funds, &input_token) {
        return Err(ContractError::InsufficientFunds {});
    }

    // generate the swap_msg
    let swap_msg = generate_swap_msg(
        deps.as_ref(),
        env.contract.address,
        input_token,
        min_output_token,
    )?;

    // save intermediate state for reply

    SWAP_REPLY_STATES.save(
        deps.storage,
        SWAP_REPLY_ID,
        &SwapMsgReplyState {
            original_sender: info.sender,
            swap_msg: swap_msg.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "trade_with_slippage_limit")
        .add_submessage(SubMsg::reply_on_success(swap_msg, SWAP_REPLY_ID)))

    // TODO: add more attributes
}

pub fn handle_swap_reply(
    _deps: DepsMut,
    msg: Reply,
    swap_msg_reply_state: SwapMsgReplyState,
) -> Result<Response, ContractError> {
    if let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result {
        let res: MsgSwapExactAmountInResponse = b.try_into().map_err(ContractError::Std)?;

        let amount = Uint128::from_str(&res.token_out_amount)?;

        let send_denom = &swap_msg_reply_state
            .swap_msg
            .routes
            .last()
            .unwrap()
            .token_out_denom;

        let bank_msg = BankMsg::Send {
            to_address: swap_msg_reply_state.original_sender.into_string(),
            amount: coins(amount.u128(), send_denom),
        };

        return Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("token_out_amount", amount));
    }

    Err(ContractError::FailedSwap {
        reason: msg.result.unwrap_err(),
    })
}
