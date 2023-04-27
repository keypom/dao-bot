/// CURRENTLY DOES NOT WORK? NOT SURE WHY
mod ext_traits;

use ext_traits::ext_dao;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::{log, near_bindgen, AccountId, Gas, env, Promise, PromiseResult, require, Balance};
use near_sdk::serde::{Deserialize, Serialize};
use std::convert::{TryFrom};
use std::collections::{HashSet};
use near_sdk::json_types::U128;

pub const XCC_GAS: Gas = Gas(20_000_000_000_000);
pub const ONETWOFIVE_NEAR: Balance = 1250000000000000000000000;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProposalKind {
    /// Add member to given role in the policy. This is short cut to updating the whole policy.
    AddMemberToRole { member_id: AccountId, role: String },
    /// Remove member to given role in the policy. This is short cut to updating the whole policy.
    RemoveMemberFromRole { member_id: AccountId, role: String },
    /// Just a signaling vote, with no execution.
    Vote,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Policy {
    /// List of roles and permissions for them in the current policy.
    pub roles: Vec<RolePermission>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize,)]
pub struct RolePermission {
    /// Name of the role to display to the user.
    pub name: String,
    /// Kind of the role: defines which users this permissions apply.
    pub kind: RoleKind
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum RoleKind {
    /// Matches everyone, who is not matched by other roles.
    Everyone,
    /// Member greater or equal than given balance. Can use `1` as non-zero balance.
    Member(U128),
    /// Set of accounts.
    Group(HashSet<AccountId>),
}

/// Injected Keypom Args struct to be sent to external contracts
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct KeypomArgs {
    pub account_id_field: Option<String>,
    pub drop_id_field: Option<String>,
    pub key_id_field: Option<String>,
    pub funder_id_field: Option<String>
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
pub enum Action {
    /// Action to add proposal. Used internally.
    AddProposal,
    // Action to remove given proposal. Used for immediate deletion in special cases.
    RemoveProposal,
    /// Vote to approve given proposal or bounty.
    VoteApprove,
    /// Vote to reject given proposal or bounty.
    VoteReject,
    /// Vote to remove given proposal or bounty (because it's spam).
    VoteRemove,
    /// Finalize proposal, called when it's expired to return the funds
    /// (or in the future can be used for early proposal closure).
    Finalize,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    drop_id_archive: LookupMap<u128, u64>,
}

/// CURRENTLY DOES NOT WORK? NOT SURE WHY

#[near_bindgen]
impl Contract {
    pub fn view_user_roles(dao_contract: String, member: String) -> Promise{
        ext_dao::ext(AccountId::try_from(dao_contract.to_string()).unwrap())
        .get_policy()
        .then(
            Self::ext(env::current_account_id())
            .view_user_roles_callback(member)
        )
    }

    pub fn view_user_roles_callback(member: String) -> Vec<String>{
        let mut user_roles = Vec::new();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pol) = near_sdk::serde_json::from_slice::<Policy>(&val) {
                    for role in pol.roles.into_iter(){
                        match role.kind{
                            RoleKind::Group(set) => {
                                // If drop funder is on council
                                if set.contains(&AccountId::try_from(member.to_string()).unwrap()){
                                    user_roles.push(role.name.clone());
                                }
                            }
                            RoleKind::Member(weight) => {
                                // If drop funder is on council
                                if weight > near_sdk::json_types::U128(0) {
                                    user_roles.push(role.name.clone());
                                }
                            }
                            RoleKind::Everyone => {
                                user_roles.push(role.name.clone());
                            }
                            
                        }
                    }
                    log!("USER ROLES: {:?}", user_roles);
                    user_roles
            } else {
                env::panic_str("ERR_WRONG_VAL_RECEIVED")
            }
        },
        PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
        } 
    }
    
}