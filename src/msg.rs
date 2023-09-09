use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub members: Vec<MemberInit>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MemberInit {
    pub address: Addr,
    pub weight: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Propose {
        title: String,
        description: String,
    },
    Vote {
        proposal_id: u64,
        approve: bool,
    },
    Execute {
        proposal_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProposal {
        proposal_id: u64,
    },
    ListProposals {},
    GetMember {
        address: Addr,
    },
    ListMembers {},
}
