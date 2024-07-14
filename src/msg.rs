use schemars::JsonSchema;
use serde::{Deserialize, Serialize,};

use cosmwasm_std::{Addr, Binary, Uint128,};

use crate::state::{State, Allocation};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub auction_admin: Addr,
    pub project_snip_contract: Addr,
    pub project_snip_hash: String,
    pub paired_snip_contract: Addr,
    pub paired_snip_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Claim {},
    EndAuction {},
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Binary,
        memo: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Deposit {},
    BeginAuction {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetState {},
    GetUserInfo {address: Addr},
    GetAllocationOptions {},
}
// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StateResponse {
    pub state: State,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllocationOptionResponse {
    pub allocations: Vec<Allocation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllocationResponse {
    pub percentages: Vec<AllocationPercentage>,
    pub allocations: Vec<Allocation>,
    pub deposit: Uint128,
}

// Messages sent to SNIP-20 contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Msg {
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    Transfer {
        recipient: Addr,
        amount: Uint128,
        padding: Option<String>,
    },
    Mint {
        recipient: Addr,
        amount: Uint128,
    },
}

impl Snip20Msg {
    pub fn register_receive(code_hash: String) -> Self {
        Snip20Msg::RegisterReceive {
            code_hash,
            padding: None, // TODO add padding calculation
        }
    }
    pub fn transfer_snip(recipient: Addr, amount: Uint128) -> Self {
        Snip20Msg::Transfer {
            recipient,
            amount,
            padding: None, // TODO add padding calculation
        }
    }
    pub fn mint_msg(recipient: Addr, amount: Uint128) -> Self {
        Snip20Msg::Mint {
            recipient,
            amount,
        }
    }
}
