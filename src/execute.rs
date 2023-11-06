use cosmwasm_std::{
    BankMsg, CosmosMsg, Coin, DepsMut, Env, MessageInfo, Response,
};

use crate::state::{State, STATE};
use crate::error::ContractError;

// Mainnet
// pub static DENOM: &str = "aarch";
// Testnet
pub static DENOM: &str = "aconst";

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    if state.gameover.clone() {
        return Err(ContractError::Unauthorized {});
    }

    // Determine if gameover, and sender will 
    // be able to call ExecuteMsg::Claim
    if state.is_expired(&env.block) {
        state.gameover = true;
    }

    let required_payment = Coin {
        denom: DENOM.to_string(),
        amount: state.min_deposit,
    };
    check_sent_required_payment(&info.funds, Some(required_payment), state.clone())?;

    if !state.gameover {
        let new_expiration: u64 = state.expiration + state.extensions;
        state.expiration = new_expiration;
        state.last_deposit = env.block.time.seconds();
        state.last_depositer = info.sender.clone();
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "execute_deposit")
        .add_attribute("depositer", info.sender)
        .add_attribute("gameover", state.gameover.to_string()))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if !state.gameover {
        return Err(ContractError::Unauthorized {});
    } else if info.sender.clone() != state.last_depositer {
        return Err(ContractError::Unauthorized {});
    }

    // Query transferrable amount
    let contract_funds = deps.querier.query_balance(env.contract.address, DENOM)?;

    // Transfer claim prizes
    let bank_transfer_msg = BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![contract_funds],
    };
    let bank_transfer: CosmosMsg = cosmwasm_std::CosmosMsg::Bank(bank_transfer_msg);

    // Reset game
    let new_expiration: u64 = env.block.time.seconds() + state.reset_length.clone();
    let state_reset = State {
        owner: state.owner,
        expiration: new_expiration,
        min_deposit: state.min_deposit,
        last_deposit: env.block.time.seconds(),
        last_depositer: info.sender.clone(),
        extensions: state.extensions,
        reset_length: state.reset_length,
        gameover: false,
    };
    STATE.save(deps.storage, &state_reset)?;

    Ok(Response::new()
        .add_attribute("action", "execute_claim")
        .add_attribute("claimant", info.sender)
        .add_message(bank_transfer))
}

pub fn check_sent_required_payment(
    sent: &[Coin],
    required: Option<Coin>,
    state: State,
) -> Result<(), ContractError> {
    if state.gameover {
        return Ok(())
    }
    if let Some(required_coin) = required {
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
                return Err(ContractError::Unauthorized {});
            }
        }
    }
    Ok(())
}