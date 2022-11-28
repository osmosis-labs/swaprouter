#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;

use crate::consts::{FORWARD_REPLY_ID, SWAP_REPLY_ID};
use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, SudoMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:crosschain-swaps";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate contract addresses and save to config
    let swap_contract = deps.api.addr_validate(&msg.swap_contract)?;
    let ibc_listeners_contract = deps.api.addr_validate(&msg.ibc_listeners_contract)?;
    let state = Config {
        swap_contract,
        ibc_listeners_contract,
    };
    CONFIG.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {}
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::OsmosisSwap {
            input_coin,
            output_denom,
            slipage,
            receiver,
            channel,
            failed_delivery,
        } => execute::swap_and_forward(
            deps,
            env.block.time,
            input_coin,
            output_denom,
            slipage,
            receiver,
            channel,
            failed_delivery,
        ),
        ExecuteMsg::Recover {} => todo!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::ReceivePacket {} => unimplemented!(),
        SudoMsg::ReceiveAck {
            channel,
            sequence,
            ack,
            success,
        } => receive_ack(deps, channel, sequence, ack, success),
        SudoMsg::ReceiveTimeout {} => unimplemented!(),
    }
}

fn receive_ack(
    deps: DepsMut,
    channel: String,
    sequence: u64,
    ack: String,
    success: bool,
) -> Result<Response, ContractError> {
    deps.api.debug(&format!(
        "received ack for packet {channel:?} {sequence:?}: {ack:?}, {success:?}"
    ));

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    deps.api
        .debug(&format!("executing crosschain reply: {reply:?}"));
    match reply.id {
        SWAP_REPLY_ID => execute::handle_swap_reply(deps, reply),
        FORWARD_REPLY_ID => execute::handle_forward_reply(deps, reply),
        id => Err(ContractError::CustomError {
            val: format!("invalid reply id: {}", id),
        }),
    }
}
