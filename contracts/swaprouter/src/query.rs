use cosmwasm_std::Env;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, StdResult};
use osmosis_std::shim::Timestamp as OsmosisTimestamp;
use osmosis_std::types::osmosis::twap::v1beta1::TwapQuerier;

use crate::msg::{GetOwnerResponse, GetRouteResponse, TestTwapResponse};
use crate::state::{ROUTING_TABLE, STATE};

pub fn query_owner(deps: Deps) -> StdResult<GetOwnerResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(GetOwnerResponse {
        owner: state.owner.into_string(),
    })
}

pub fn query_route(
    deps: Deps,
    input_token: &str,
    output_token: &str,
) -> StdResult<GetRouteResponse> {
    let route = ROUTING_TABLE.load(deps.storage, (input_token, output_token))?;

    Ok(GetRouteResponse { pool_route: route })
}

pub fn test_twap(deps: Deps, env: Env) -> StdResult<TestTwapResponse> {
    let start_time = env.block.time.minus_seconds(300);
    let start_time = OsmosisTimestamp {
        seconds: start_time.seconds() as i64,
        nanos: 0_i32,
    };

    let twap = TwapQuerier::new(&deps.querier)
        .arithmetic_twap_to_now(
            1,
            "uosmo".to_string(),
            "uion".to_string(),
            Some(start_time.clone()),
        )?
        .arithmetic_twap;

    Ok(TestTwapResponse { price: twap })
}
