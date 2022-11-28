#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::consts::{FORWARD_REPLY_ID, SWAP_REPLY_ID};
use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, Status, CONFIG, INFLIGHT_PACKETS, RECOVERY_STATES};

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
    info: MessageInfo,
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
        ExecuteMsg::Recover {} => execute::recover(deps, info.sender),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Recoverable { addr } => {
            to_binary(&RECOVERY_STATES.may_load(deps.storage, &addr)?)
        }
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
    let response = Response::new().add_attribute("contract", "crosschain_swaps");
    let recovery = INFLIGHT_PACKETS.may_load(deps.storage, (&channel, sequence))?;
    INFLIGHT_PACKETS.remove(deps.storage, (&channel, sequence));
    let Some(mut recovery) = recovery else {
      return Ok(response.add_attribute("msg", "received unexpected ack"))
    };

    if success {
        return Ok(response.add_attribute("msg", "packet successfully delviered"));
    }

    let recovery_addr = recovery.recovery_addr.clone();
    RECOVERY_STATES.update(deps.storage, &recovery_addr, |recoveries| {
        recovery.status = Status::AckFailure;
        let Some(mut recoveries) = recoveries else {
            return Ok::<_, ContractError>(vec![recovery])
        };
        recoveries.push(recovery);
        Ok(recoveries)
    })?;

    Ok(response
        .add_attribute("msg", "Recovery Stored")
        .add_attribute("reecovery_addr", recovery_addr))
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
