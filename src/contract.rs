use cosmwasm_std::{
    entry_point, BankMsg, SubMsg, Coin, Coins, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, StdError
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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Propose { title, description, recipient, amount } => execute_propose(deps, info, title, description),
        ExecuteMsg::Vote { proposal_id, approve } => execute_vote(deps, info, proposal_id, approve),
        ExecuteMsg::Execute { proposal_id } => execute_execute(deps, env, proposal_id),  // Add env here
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
        amount: Uint128::zero(), //TODO: set to param at instantiate time
        recipient: info.sender.clone(),
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
    env: Env,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS.load(deps.storage, &proposal_id.to_string())?;

    if proposal.executed {
        return Err(ContractError::AlreadyExecuted {});
    }

    if proposal.votes_for > proposal.votes_against {
        let recipient = &proposal.recipient;
        let amount = &proposal.amount;

        let current_balance = Uint128::zero();

        if current_balance < *amount {
            return Err(ContractError::Std(StdError::generic_err("Insufficient funds")));
        }

        let transfer = BankMsg::Send {
            to_address: recipient.clone().into(),
            amount: vec![Coin {
                denom: "udevcore".to_string(),
                amount: amount.clone(),
            }],
        };

        proposal.executed = true;
        PROPOSALS.save(deps.storage, &proposal_id.to_string(), &proposal)?;

        let cosmos_msg = cosmwasm_std::CosmosMsg::Bank(transfer);

        return Ok(Response::new()
            .add_message(cosmos_msg) // TODO: use coreum message instead?
            .add_attribute("method", "execute_execute")
            .add_attribute("recipient", recipient.to_string())
            .add_attribute("amount", amount.to_string()));
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
    use cosmwasm_std::{from_binary, Addr, Uint128};
    use crate::state::Member;


    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        
        let members = vec![
            Member {
                address: Addr::unchecked("addr1"),
                weight: Uint128::from(10_u128),
            },
            Member {
                address: Addr::unchecked("addr2"),
                weight: Uint128::from(20_u128),

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
            Member {
                address: Addr::unchecked("addr1"),
                weight: Uint128::from(10_u128),
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
            amount: Some(Uint128::from(100_u128)),
            recipient: Some(Addr::unchecked("recipient_address")),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn vote_for_proposal() {
        let mut deps = mock_dependencies();

        let members = vec![
            Member {
                address: Addr::unchecked("addr1"),
                weight: Uint128::from(10_u128),
            },
        ];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Propose
        let info = mock_info("addr1", &[]);
        let proposal_msg = ExecuteMsg::Propose {
            title: "Some Title".to_string(),
            description: "Some Description".to_string(),
            amount: Some(Uint128::from(100_u128)),
            recipient: Some(Addr::unchecked("recipient_address")),
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
            Member {
                address: Addr::unchecked("addr1"),
                weight: Uint128::from(10_u128),
            },
        ];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("addr1", &[]);
        let proposal_msg = ExecuteMsg::Propose {
            title: "Another Title".to_string(),
            description: "Another Description".to_string(),
            amount: Some(Uint128::from(100_u128)),
            recipient: Some(Addr::unchecked("recipient_address")),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), proposal_msg).unwrap();

        let vote_msg = ExecuteMsg::Vote {
            proposal_id: 0,
            approve: true,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), vote_msg).unwrap();

        let exec_msg = ExecuteMsg::Execute { proposal_id: 0 };
        let res = execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
