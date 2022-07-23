use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SetAdmin {
        admin: HumanAddr,
    },
    WriteResult {
        code_id: u16,
        repo: String,
        commit_hash: String,
        method: String,
        verified: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CheckCodeId { code_id: u16 },
    CheckAllVerified {},
}
