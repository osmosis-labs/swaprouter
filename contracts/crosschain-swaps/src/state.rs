use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use swaprouter::msg::ExecuteMsg as SwapRouterExecute;

#[cw_serde]
pub struct Config {
    pub swap_contract: Addr,
}

#[cw_serde]
pub struct ForwardTo {
    pub channel: String,
    pub receiver: Addr,
}

#[cw_serde]
pub struct SwapMsgReplyState {
    pub swap_msg: SwapRouterExecute,
    pub block_time: Timestamp,
    pub forward_to: ForwardTo,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SWAP_REPLY_STATES: Map<u64, SwapMsgReplyState> = Map::new("swap_reply_states");
