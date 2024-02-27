use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub archid_registry: Option<Addr>,
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
    Configure {
        msg: ConfigureMsg,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Game {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigureMsg {
    pub owner: Option<Addr>,
    pub archid_registry: Option<Addr>,
    pub expiration: Option<u64>,
    pub min_deposit: Option<Uint128>,
    pub extensions: Option<u64>,
    pub stale: Option<u64>,
    pub reset_length: Option<u64>,
}
