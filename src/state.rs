use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};

use secret_toolkit_storage::{Keymap, Item};
use crate::msg::{AllocationPercentage};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub auction_admin: Addr,
    pub project_snip_contract: Addr,
    pub project_snip_hash: String,
    pub paired_snip_contract: Addr,
    pub paired_snip_hash: String,
    pub auction_amount: Uint128,
    pub total_deposits: Uint128,
    pub auction_active: bool,
}

pub static STATE: Item<State> = Item::new(b"state");

pub static DEPOSITS: Keymap<Addr, Uint128> = Keymap::new(b"deposit_amounts");





