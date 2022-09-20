use std::ops::{Div, Mul};

use cosmwasm_std::{Addr, Coin, Decimal, Deps, Timestamp, Uint128};
use osmosis_std::shim::Timestamp as OsmosisTimestamp;
use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgSwapExactAmountIn, QueryTotalPoolLiquidityRequest, SwapAmountInRoute,
};
use osmosis_std::types::osmosis::twap::v1beta1::TwapQuerier;

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
        .query(&deps.querier)?
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

pub fn calculate_min_output_from_twap(
    deps: Deps,
    input_token: Coin,
    output_denom: String,
    now: Timestamp,
    percentage_impact: Decimal,
) -> Result<Coin, ContractError> {
    // get trade route
    let route = ROUTING_TABLE.load(deps.storage, (&input_token.denom, &output_denom))?;
    if route.is_empty() {
        return Err(ContractError::InvalidPoolRoute {
            reason: format!("No route foung for {} -> {output_denom}", input_token.denom),
        });
    }

    let percentage = percentage_impact.div(Uint128::new(100));

    let mut twap_price: Decimal = Decimal::one();
    let quote_denom = input_token.denom;

    let start_time = now.minus_seconds(300);
    let start_time = OsmosisTimestamp {
        seconds: start_time.seconds() as i64,
        nanos: 0_i32,
    };

    for route_part in route {
        let twap = TwapQuerier::new(&deps.querier)
            .arithmetic_twap_to_now(
                route_part.pool_id,
                quote_denom.clone(),
                route_part.token_out_denom,
                Some(start_time.clone()),
            )?
            .arithmetic_twap;

        let twap: Decimal = twap.parse().map_err(|_e| ContractError::CustomError {
            val: "Invalid twap value received from the chain".to_string(),
        })?;
        twap_price =
            twap_price
                .checked_mul(twap.into())
                .map_err(|_e| ContractError::CustomError {
                    val: format!("Invalid value for twap price: {twap_price} * {twap}"),
                })?;
    }

    twap_price = twap_price - twap_price.mul(percentage);
    let min_out: Uint128 = input_token.amount.mul(twap_price);

    Ok(Coin::new(min_out.into(), output_denom))
}
