use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub votes_for: Uint128,
    pub votes_against: Uint128,
    pub executed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Member {
    pub address: Addr,
    pub weight: Uint128,
}

pub const STATE: Item<()> = Item::new("state");
pub const PROPOSALS: Map<&str, Proposal> = Map::new("proposals");
pub const MEMBERS: Map<&str, Member> = Map::new("members");
