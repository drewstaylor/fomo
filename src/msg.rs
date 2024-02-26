use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub expiration: u64,
    pub min_deposit: Uint128,
    pub extensions: u64,
    pub stale: u64,
    pub reset_length: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit {},
    Claim {},
    UnlockStale {},
    // Admin only
    Pause {},
    Unpause {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Game {},
}
