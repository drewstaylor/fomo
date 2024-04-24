use cosmwasm_std::{
    BankMsg, CosmosMsg, Coin, DepsMut, Env, MessageInfo, QueryRequest, Response,
    to_binary, WasmQuery
};

use archid_registry::msg::{QueryMsg as QueryMsgArchid, ResolveAddressResponse};

use crate::contract::DENOM;
use crate::msg::{ConfigureMsg};
use crate::state::{ARCHID, STATE};
use crate::error::ContractError;

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Game play must not be paused for upgrades
    if state.is_paused() {
        return Err(ContractError::Paused {});
    }

    // Game must be active
    if state.is_expired(&env.block) {
        return Err(ContractError::Gameover {});
    }

    // Sender should own an ArchID
    if let Some(contract_addr) = ARCHID.may_load(deps.storage)? {
        let query_msg: archid_registry::msg::QueryMsg = QueryMsgArchid::ResolveAddress { 
            address: info.sender.clone(),
        };
        let request = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&query_msg).unwrap(),
        });
        let response: ResolveAddressResponse = deps.querier.query(&request)?;
        let valid_archids: Vec<String> = response.names.unwrap_or(vec![]);
        if valid_archids.is_empty() {
            return Err(ContractError::NoArchid {});
        }
    }

    // Sender must have sent correct funds
    let required_payment = Coin {
        denom: DENOM.to_string(),
        amount: state.min_deposit,
    };
    check_sent_required_payment(&info.funds, Some(required_payment))?;

    // Update state with deposit parameters
    state.expiration += state.extensions;
    state.last_deposit = env.block.time.seconds();
    state.last_depositor = info.sender.clone();
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_deposit")
        .add_attribute("round", state.round.to_string())
        .add_attribute("depositor", info.sender))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Game play must not be paused
    if state.is_paused() {
        return Err(ContractError::Paused {});
    }

    // Game must be ended
    if !state.is_expired(&env.block) {
        return Err(ContractError::Gameover {});
    }
    
    // Caller must be winner
    if info.sender != state.last_depositor {
        return Err(ContractError::Unauthorized {});
    }

    // Query transferrable amount
    let contract_funds = deps.querier.query_balance(env.contract.address, DENOM)?;

    // Transfer claim prizes
    let bank_transfer_msg = BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![contract_funds],
    };
    let bank_transfer: CosmosMsg = CosmosMsg::Bank(bank_transfer_msg);

    // Reset game
    let won_round = state.round.to_string();
    state.reset(env.block.time.seconds(), &info);

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_claim")
        .add_attribute("winner", info.sender)
        .add_attribute("round", won_round)
        .add_message(bank_transfer))
}

pub fn execute_unlock_stale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Game must not be paused for upgrades
    if state.is_paused() {
        return Err(ContractError::Paused {});
    }

    // Game must be ended
    if !state.is_expired(&env.block) {
        return Err(ContractError::Gameover {});
    }

    // Game must be stale
    if !state.is_stale(&env.block) {
        return Err(ContractError::NotStale {});
    }

    // Reset game, retaining the current prize pool
    let skipped_round = state.round.to_string();
    state.reset(env.block.time.seconds(), &info);

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_unlock_stale")
        .add_attribute("round", skipped_round))
}

// Pause game for upgrade (admin only)
pub fn execute_pause(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Must not be paused already
    if state.is_paused() {
        return Err(ContractError::Paused {});
    }

    // Only admin can pause
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let paused_at: u64 = env.block.time.seconds();
    state.paused = Some(paused_at);
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_pause")
        .add_attribute("paused_at", paused_at.to_string()))
}

// Resume / unpause game play after conducting upgrades (admin only)
pub fn execute_unpause(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Game must be paused
    if !state.is_paused() {
        return Err(ContractError::InvalidInput {});
    }

    // Only Admin can unpause game
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Unpause game
    let unpaused_at: u64 = env.block.time.seconds();
    let paused_duration: u64 = unpaused_at - state.paused.unwrap_or(unpaused_at);
    let new_expiration: u64 = state.expiration + paused_duration;
    state.expiration = new_expiration;
    state.paused = None;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_unpause")
        .add_attribute("unpaused_at", unpaused_at.to_string())
        .add_attribute("time_paused", paused_duration.to_string())
        .add_attribute("expiration", new_expiration.to_string()))
}

// Reconfigure game parameters (admin only)
pub fn execute_configure(
    deps: DepsMut,
    info: MessageInfo,
    msg: ConfigureMsg,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    // Only Admin can unpause game
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Reconfiguration must change at least 1 value
    if msg.owner.is_none() 
        && msg.archid_registry.is_none()
        && msg.expiration.is_none() 
        && msg.min_deposit.is_none()
        && msg.extensions.is_none()
        && msg.stale.is_none()
        && msg.reset_length.is_none() {
            return Err(ContractError::InvalidInput {});
        }

    // Game settings
    if let Some(new_owner) = msg.owner {
        state.owner = new_owner;
    }
    if let Some(new_expiration) = msg.expiration {
        state.expiration = new_expiration;
    }
    if let Some(new_min_deposit) = msg.min_deposit {
        state.min_deposit = new_min_deposit;
    }
    if let Some(new_extensions) = msg.extensions {
        state.extensions = new_extensions;
    }
    if let Some(new_stale) = msg.stale {
        state.stale = new_stale;
    }
    if let Some(new_reset_length) = msg.reset_length {
        state.reset_length = new_reset_length;
    }

    // ArchID settings
    if let Some(new_archid_registry) = msg.archid_registry {
        ARCHID.save(deps.storage, &new_archid_registry)?;
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_configure"))
}

pub fn check_sent_required_payment(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<(), ContractError> {
    if let Some(ref required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                coin.denom == required_coin.denom && coin.amount.u128() >= required_amount
            });

            if sent_sufficient_funds {
                return Ok(());
            } else {
                return Err(ContractError::InsufficientFunds { required });
            }
        }
    }
    Ok(())
}