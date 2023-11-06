use cosmwasm_std::{Deps, StdResult};
use crate::state::{State, STATE};

pub fn query_game(deps: Deps) -> StdResult<State> {
    let gamestate: State = STATE.load(deps.storage)?;
    Ok(gamestate)
}