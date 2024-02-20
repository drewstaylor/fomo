use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input")]
    InvalidInput {},

    #[error("Insufficient funds")]
    InsufficientFunds {
        required: Option<Coin>,
    },
    
    #[error("Gameplay will resume when last depositor claims their prize")]
    Gameover {},
}
