use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use swaprouter::msg::ExecuteMsg as SwapRouterExecute;

use crate::msg::Recovery;

#[cw_serde]
pub struct Config {
    pub swap_contract: Addr,
    pub track_ibc_callbacks: bool,
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

/// A transfer packet sent by this contract that is expected to be received but
/// needs to be tracked in case it is not
#[cw_serde]
pub struct IBCTransfer {
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

/// In-Flight packets by (source_channel_id, sequence)
pub const INFLIGHT_PACKETS: Map<(&str, u64), IBCTransfer> = Map::new("inflight");

/// Recovery. This tracks any recovery that an addr can execute.
pub const RECOVERY_STATES: Map<&Addr, Vec<IBCTransfer>> = Map::new("recovery");
