use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountIn, SwapAmountInRoute};

use cosmwasm_std::{Addr, Coin, Deps};

use crate::{
    state::{ROUTING_TABLE, STATE},
    ContractError,
};

pub fn check_is_contract_owner(deps: Deps, sender: Addr) -> Result<(), ContractError> {
    let config = STATE.load(deps.storage).unwrap();
    if config.owner != sender {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_pool_route(
    _deps: Deps,
    _input_denom: String,
    output_denom: String,
    pool_route: Vec<SwapAmountInRoute>,
) -> Result<(), ContractError> {
    // make sure the final route output asset is the same as the expected output_denom
    if pool_route.last().unwrap().token_out_denom != output_denom {
        return Result::Err(ContractError::InvalidPoolRoute {});
    }

    Ok(())
}

pub fn generate_swap_msg(
    deps: Deps,
    sender: Addr,
    input_token: Coin,
    min_output_token: Coin,
) -> Result<MsgSwapExactAmountIn, ContractError> {
    // get trade route
    let route = ROUTING_TABLE.load(deps.storage, (&input_token.denom, &min_output_token.denom))?;

    // convert input coin to sdk coin stuct
    let sdk_input_coin = osmosis_std::types::cosmos::base::v1beta1::Coin {
        denom: input_token.denom,
        amount: input_token.amount.to_string(),
    };

    Ok(MsgSwapExactAmountIn {
        sender: sender.into_string(),
        routes: route,
        token_in: Some(sdk_input_coin),
        token_out_min_amount: min_output_token.amount.to_string(),
    })
}
