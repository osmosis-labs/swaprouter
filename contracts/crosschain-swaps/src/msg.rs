use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};
use swaprouter::msg::Slipage;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub swap_contract: String,
}

#[cw_serde]
pub struct Recovery {
    pub recovery_addr: Addr,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    OsmosisSwap {
        input_coin: Coin,
        output_denom: String,
        slipage: Slipage,
        receiver: Addr,
        channel: String,
        failed_delivery: Option<Recovery>,
    },
    Recover {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<crate::state::RecoveryState>)]
    Recoverable { addr: Addr },
}

// tmp structure for crosschain response
#[cw_serde]
pub struct CrosschainSwapResponse {
    pub msg: String,
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
pub enum SudoMsg {
    ReceivePacket {},
    ReceiveAck {
        channel: String,
        sequence: u64,
        ack: String,
        success: bool,
    },
    ReceiveTimeout {},
}

// Copying this temporarily
#[cw_serde]
pub enum ListenersMsg {
    Subscribe {
        channel: String,
        sequence: u64,
        event: EventType,
    },
}

// Copying this while the contract is not importable
#[cw_serde]
pub enum EventType {
    Acknowledgement,
    Timeout,
}
