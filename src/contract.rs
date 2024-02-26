#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, 
    StdResult,
};
use cw2::{get_contract_version, set_contract_version};

use crate::execute::{
    execute_claim, execute_deposit, execute_pause, execute_unlock_stale, execute_unpause
};
use crate::query::{query_game};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{State, STATE};
use crate::error::ContractError;

// Mainnet
// pub static DENOM: &str = "aarch";
// Testnet
pub static DENOM: &str = "aconst";

// version info for migration info
const CONTRACT_NAME: &str = "network-wars";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        expiration: msg.expiration + env.block.time.seconds(),
        min_deposit: msg.min_deposit,
        last_deposit: env.block.time.seconds(),
        last_depositor: info.sender.clone(),
        extensions: msg.extensions,
        stale: msg.stale,
        reset_length: msg.reset_length,
        round: 1_u64,
        paused: None,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => execute_deposit(deps, env, info),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::UnlockStale {} => execute_unlock_stale(deps, env, info),
        // Admin only
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Game {} => to_binary(&query_game(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // Game play must be paused for upgrade
    let state = STATE.load(deps.storage)?;
    if !state.is_paused() {
        return Err(ContractError::Unauthorized {});
    }

    let original_version = get_contract_version(deps.storage)?;
    let name = CONTRACT_NAME.to_string();
    let version = CONTRACT_VERSION.to_string();
    if original_version.contract != name {
        return Err(ContractError::InvalidInput {});
    }
    if original_version.version >= version {
        return Err(ContractError::InvalidInput {});
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, Uint128};

    #[test]
    fn can_instantiate() {
        let mut deps = mock_dependencies();

        let res = instantiate_contract(deps.as_mut());
        assert_eq!(0, res.messages.len());

        let owner = &res
            .attributes
            .iter()
            .find(|a| a.key == "owner")
            .unwrap()
            .value;
        assert_eq!("creator", owner);
    }

    fn instantiate_contract(deps: DepsMut) -> Response {
        let env = mock_env();
        let extends: u64 = 1000;
        let reset: u64 = extends * 5;
        let expires: u64 = env.block.time.seconds() + extends.clone();
        
        let msg = InstantiateMsg {
            expiration: expires,
            min_deposit: Uint128::from(1000000u128),
            extensions: extends,
            stale: reset.clone(),
            reset_length: reset,
        };
        let info = mock_info("creator", &coins(1000, "token"));
        instantiate(deps, mock_env(), info, msg).unwrap()
    }
}
