use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use swaprouter::msg::ExecuteMsg as SwapRouterExecute;

use crate::msg::Recovery;

#[cw_serde]
pub struct Config {
    pub swap_contract: Addr,
}

#[cw_serde]
pub struct ForwardTo {
    pub channel: String,
    pub receiver: Addr,
    pub failed_delivery: Option<Recovery>,
}

#[cw_serde]
pub struct SwapMsgReplyState {
    pub swap_msg: SwapRouterExecute,
    pub contract_addr: Addr,
    pub block_time: Timestamp,
    pub forward_to: ForwardTo,
}

#[cw_serde]
pub struct ForwardMsgReplyState {
    pub channel_id: String,
    pub to_address: String,
    pub amount: u128,
    pub denom: String,
    pub failed_delivery: Option<Recovery>,
}

#[cw_serde]
pub enum Status {
    Sent,
    AckSuccess,
    AckFailure,
    TimedOut,
}

#[cw_serde]
pub struct RecoveryState {
    pub recovery_addr: Addr,
    pub channel_id: String,
    pub sequence: u64,
    pub amount: u128,
    pub denom: String,
    pub status: Status,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SWAP_REPLY_STATES: Item<SwapMsgReplyState> = Item::new("swap_reply_states");
pub const FORWARD_REPLY_STATES: Item<ForwardMsgReplyState> = Item::new("forward_reply_states");

// Recovery
pub const RECOVERY_STATES: Map<&Addr, Vec<RecoveryState>> = Map::new("recovery");

// In-Flight packets by (channel_id, sequence)
pub const INFLIGHT_PACKETS: Map<(&str, u64), RecoveryState> = Map::new("inflight");
