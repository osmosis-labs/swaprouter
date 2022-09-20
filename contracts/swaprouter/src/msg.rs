use cosmwasm_std::{Coin, Decimal, Uint128};
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Slipage {
    MaxPriceImpactPercentage(Decimal),
    MinOutputAmount(Uint128),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetRoute {
        input_denom: String,
        output_denom: String,
        pool_route: Vec<SwapAmountInRoute>,
    },
    Swap {
        input_coin: Coin,
        output_denom: String,
        slipage: Slipage,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetOwner {},
    TestTwap {},
    GetRoute {
        input_denom: String,
        output_denom: String,
    },
}

// Response for GetOwner query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetOwnerResponse {
    pub owner: String,
}

// Response for GetRoute query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetRouteResponse {
    pub pool_route: Vec<SwapAmountInRoute>,
}

// Response for GetRoute query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TestTwapResponse {
    pub price: String,
}
