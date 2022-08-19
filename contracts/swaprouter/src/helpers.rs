use cosmwasm_std::{Addr, Coin, Decimal, Deps, Timestamp};
use osmosis_std::types::osmosis::{
    self,
    gamm::v1beta1::{
        MsgSwapExactAmountIn, QueryTotalPoolLiquidityRequest, QueryTotalPoolLiquidityResponse,
        SwapAmountInRoute,
    },
};

use osmo_bindings::{OsmosisQuerier, OsmosisQuery};

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
    deps: Deps,
    input_denom: String,
    output_denom: String,
    pool_route: Vec<SwapAmountInRoute>,
) -> Result<(), ContractError> {
    let mut current_denom = input_denom;

    // make sure that this route actually works
    for route_part in &pool_route {
        let liquidity = QueryTotalPoolLiquidityRequest {
            pool_id: route_part.pool_id,
        }
        .query(deps.querier)?
        .liquidity;

        if !liquidity.iter().any(|coin| coin.denom == current_denom) {
            return Result::Err(ContractError::InvalidPoolRoute {
                reason: format!(
                    "denom {} is not in pool id {}",
                    current_denom, route_part.pool_id
                ),
            });
        }

        if !liquidity
            .iter()
            .any(|coin| coin.denom == route_part.token_out_denom)
        {
            return Result::Err(ContractError::InvalidPoolRoute {
                reason: format!(
                    "denom {} is not in pool id {}",
                    current_denom, route_part.pool_id
                ),
            });
        }

        current_denom = route_part.token_out_denom.clone();
    }

    // make sure the final route output asset is the same as the expected output_denom
    if current_denom != output_denom {
        return Result::Err(ContractError::InvalidPoolRoute {
            reason: "last denom doesn't match".to_string(),
        });
    }

    Ok(())
}

pub fn get_multihop_twap(
    deps: Deps<OsmosisQuery>,
    input_denom: String,
    output_denom: String,
    start_time: Timestamp,
) -> Result<Decimal, ContractError> {
    let route = ROUTING_TABLE.load(deps.storage, (&input_denom, &output_denom))?;

    let querier = OsmosisQuerier::new(&deps.querier);
    let start_time_unix: i64 = start_time.nanos() as i64;

    let mut twap_price: Decimal = Decimal::one();
    let quote_denom = input_denom;

    for route_part in route {
        let response = querier
            .arithmetic_twap_to_now(
                route_part.pool_id,
                quote_denom.clone(),
                route_part.token_out_denom,
                start_time_unix,
            )
            .unwrap(); // TODO fix unwrap

        twap_price = twap_price.checked_mul(response.twap).unwrap(); // TODO fix unwrap
    }

    Ok(twap_price)
}

pub fn generate_swap_msg(
    deps: Deps,
    sender: Addr,
    input_token: Coin,
    min_output_token: Coin,
) -> Result<MsgSwapExactAmountIn, ContractError> {
    // get trade route
    let route = ROUTING_TABLE.load(deps.storage, (&input_token.denom, &min_output_token.denom))?;

    Ok(MsgSwapExactAmountIn {
        sender: sender.into_string(),
        routes: route,
        token_in: Some(input_token.into()),
        token_out_min_amount: min_output_token.amount.to_string(),
    })
}
