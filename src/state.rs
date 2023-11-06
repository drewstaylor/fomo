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
    pub last_depositer: Addr,
    pub extensions: u64,
    pub gameover: bool,
}
impl State {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        Expiration::AtTime(Timestamp::from_seconds(self.expiration)).is_expired(block)
    }
}

pub const STATE: Item<State> = Item::new("state");
