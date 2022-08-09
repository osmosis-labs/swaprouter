use std::marker::PhantomData;

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    to_binary, Coin, ContractResult, OwnedDeps, SystemError,
    SystemResult, Uint128,
};
use osmo_bindings::{OsmosisQuery, PoolStateResponse};

pub fn mock_deps_with_custom_querier(
) -> OwnedDeps<MockStorage, MockApi, MockQuerier<OsmosisQuery>, OsmosisQuery> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(&[]).with_custom_handler(|query| match query {
            OsmosisQuery::FullDenom {
                creator_addr: _,
                subdenom: _,
            } => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "custom".to_string(),
            }),
            OsmosisQuery::PoolState { id } => SystemResult::Ok(ContractResult::Ok(
                to_binary(&PoolStateResponse {
                    assets: vec![
                        Coin {
                            denom: "uosmo".to_string(),
                            amount: Uint128::from(1000000000u128),
                        },
                        Coin {
                            denom: "uion".to_string(),
                            amount: Uint128::from(1000000000u128),
                        },
                    ],
                    shares: Coin {
                        denom: "gamm/pool/".to_string() + &id.to_string(),
                        amount: Uint128::from(1000000000u128),
                    },
                })
                .unwrap(),
            )),
            OsmosisQuery::SpotPrice {
                swap: _,
                with_swap_fee: _,
            } => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "custom".to_string(),
            }),
            OsmosisQuery::EstimateSwap {
                sender: _,
                first: _,
                route: _,
                amount: _,
            } => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: "custom".to_string(),
            }),
        }),
        custom_query_type: PhantomData,
    }
}
