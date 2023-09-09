use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, StdError
};
use cw2::set_contract_version;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Proposal, Member, PROPOSALS, MEMBERS};

const CONTRACT_NAME: &str = "grant-dao";
const CONTRACT_VERSION: &str = "0.1.0";

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Invalid input")]
    InvalidInput(String),
    #[error("Already Executed")]
    AlreadyExecuted {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    for member in msg.members {
        MEMBERS.save(deps.storage, member.address.as_str(), &Member {
            address: member.address.clone(),
            weight: Uint128::from(member.weight),
        })?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Propose { title, description } => execute_propose(deps, info, title, description),
        ExecuteMsg::Vote { proposal_id, approve } => execute_vote(deps, info, proposal_id, approve),
        ExecuteMsg::Execute { proposal_id } => execute_execute(deps, proposal_id),
    }
}

fn execute_propose(
    deps: DepsMut,
    info: MessageInfo,
    title: String,
    description: String,
) -> Result<Response, ContractError> {

    
    let sender_addr = info.sender.as_str();
    let member_opt = MEMBERS.load(deps.storage, sender_addr);

    if member_opt.is_err() {
        return Err(ContractError::Unauthorized {});
    }

    let proposal = Proposal {
        id: 0, 
        title,
        description,
        votes_for: Uint128::zero(),
        votes_against: Uint128::zero(),
        executed: false,
    };

    PROPOSALS.save(deps.storage, &proposal.id.to_string(), &proposal)?;

    Ok(Response::default())
}

fn execute_vote(
    deps: DepsMut,
    info: MessageInfo,
    proposal_id: u64,
    approve: bool,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.as_str();
    let member_opt = MEMBERS.load(deps.storage, sender_addr); 

    if member_opt.is_err() {
        return Err(ContractError::Unauthorized {});
    }

    let member = member_opt.unwrap();

    let mut proposal = PROPOSALS.load(deps.storage, &proposal_id.to_string())?;

    if approve {
        proposal.votes_for += member.weight;
    } else {
        proposal.votes_against += member.weight;
    }

    PROPOSALS.save(deps.storage, &proposal_id.to_string(), &proposal)?;

    Ok(Response::default())
}

fn execute_execute(
    deps: DepsMut,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS.load(deps.storage, &proposal_id.to_string())?;

    if proposal.executed {
        return Err(ContractError::AlreadyExecuted {});
    }

    if proposal.votes_for > proposal.votes_against {
        proposal.executed = true;
        PROPOSALS.save(deps.storage, &proposal_id.to_string(), &proposal)?;
    }

    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Err(StdError::generic_err("Not implemented")) 
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr};
    use crate::msg::MemberInit;


    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        
        let members = vec![
            MemberInit {
                address: Addr::unchecked("addr1"),
                weight: u128::from(10u128),
            },
            MemberInit {
                address: Addr::unchecked("addr2"),
                weight: u128::from(20u128),
            },
        ];
        let msg = InstantiateMsg { members }; 
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proposal_creation() {
        let mut deps = mock_dependencies();

        let members = vec![
            MemberInit {
                address: Addr::unchecked("addr1"),
                weight: u128::from(10u128),
            },
        ];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Propose
        let info = mock_info("addr1", &[]);
        let msg = ExecuteMsg::Propose {
            title: "Test Proposal".to_string(),
            description: "Description for test".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn vote_for_proposal() {
        let mut deps = mock_dependencies();

        let members = vec![
            MemberInit {
                address: Addr::unchecked("addr1"),
                weight: u128::from(10u128),
            },
        ];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Propose
        let info = mock_info("addr1", &[]);
        let proposal_msg = ExecuteMsg::Propose {
            title: "Test Proposal".to_string(),
            description: "Description for test".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), proposal_msg).unwrap();

        let vote_msg = ExecuteMsg::Vote {
            proposal_id: 0,
            approve: true,
        };

        let res = execute(deps.as_mut(), mock_env(), info, vote_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn execute_proposal() {
        let mut deps = mock_dependencies();

        let members = vec![
            MemberInit {
                address: Addr::unchecked("addr1"),
                weight: u128::from(10u128).into(),
            },
        ];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("addr1", &[]);
        let proposal_msg = ExecuteMsg::Propose {
            title: "Test Proposal".to_string(),
            description: "Description for test".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), proposal_msg).unwrap();

        let vote_msg = ExecuteMsg::Vote {
            proposal_id: 0,
            approve: true,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), vote_msg).unwrap();

        let exec_msg = ExecuteMsg::Execute { proposal_id: 0 };
        let res = execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
