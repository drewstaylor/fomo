use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128};
use cw_storage_plus::Item;
use cw_utils::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub expiration: u64,
    pub min_deposit: Uint128,
    pub last_deposit: u64,
    pub last_depositor: Addr,
    pub extensions: u64,
    pub stale: u64,
    pub reset_length: u64,
    pub round: u64,
    pub paused: Option<u64>,
}
impl State {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        Expiration::AtTime(Timestamp::from_seconds(self.expiration)).is_expired(block)
    }
    pub fn is_stale(&self, block: &BlockInfo) -> bool {       
        let stale = self.expiration + self.stale;
        Expiration::AtTime(Timestamp::from_seconds(stale)).is_expired(block)
    }
    pub fn is_paused(&self) -> bool {
        self.paused.is_some()
    }
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Archid {
    pub registry: Option<Addr>,
    pub cw721: Option<Addr>,
}

pub const ARCHID: Item<Archid> = Item::new("archid");