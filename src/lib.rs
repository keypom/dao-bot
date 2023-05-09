
mod ext_traits;

use ext_traits::ext_dao;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId, Gas, env, Promise, PromiseResult, require, Balance};
use near_sdk::serde::{Deserialize, Serialize};
use std::convert::{TryFrom};
use std::collections::{HashSet};
use near_sdk::json_types::U128;

pub const XCC_GAS: Gas = Gas(20_000_000_000_000);
// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

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
    keypom_contract: String
}

impl Default for Contract{
    fn default() -> Self{
        Self{
            keypom_contract: "v2.keypom.testnet".to_string()
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {

    #[payable]
    pub fn new_proposal(&mut self, dao_contract: String, keypom_args: KeypomArgs, funder: String, proposal: ProposalInput) {
        // Ensure Keypom called this function 
        // When running ava test script, comment out predecessor check. Otherwise, will fail
        require!(env::predecessor_account_id() == AccountId::try_from(self.keypom_contract.clone()).unwrap(), "KEYPOM MUST BE PREDECESSOR, CHECK REQUIRED VERSION USING view_keypom_contract");
        require!(keypom_args.funder_id_field == Some("funder".to_string()) && keypom_args.account_id_field == Some("proposal.kind.AddMemberToRole.member_id".to_string()), "KEYPOM MUST SEND THESE ARGS");
        
        // Ensure enough attached deposit was added to add the proposal
        require!(env::attached_deposit() >= SPUTNIK_PROPOSAL_DEPOSIT, "ATTACH MORE NEAR, AT LEAST 0.1 $NEAR");
        
        // Begin auto-registration
        ext_dao::ext(AccountId::try_from(dao_contract.clone().to_string()).unwrap())
        .get_policy()
        .then(
            Self::ext(env::current_account_id())
            .internal_get_roles_callback(funder, proposal, dao_contract)
        );
    } 

    
    // Roles callback, parse and return council role(s)
    #[private]
    pub fn internal_get_roles_callback(&mut self, funder: String, proposal: ProposalInput, dao_contract: String){
        // Receive get_policy promise, parse it and see if funder is on DAO council
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pol) = near_sdk::serde_json::from_slice::<Policy>(&val) {
                    // Trying to collect all roles with name council from policy
                    let members = pol.roles.into_iter()
                    .filter(|role| role.name == "council".to_string())
                    .collect::<Vec<RolePermission>>()
                    .into_iter()
                    .nth(0)
                    .unwrap().kind;

                    // See if funder is in Council group
                    match members{
                        RoleKind::Group(set) => {
                            if set.contains(&AccountId::try_from(funder.to_string()).unwrap()){
                                // Add proposal to register member if funder is on council
                                ext_dao::ext(AccountId::try_from(dao_contract.clone().to_string()).unwrap())
                                .with_attached_deposit(SPUTNIK_PROPOSAL_DEPOSIT)
                                .add_proposal(proposal)
                                .then(
                                    Self::ext(env::current_account_id())
                                    .callback_new_proposal(dao_contract)
                                );   
                            }
                            else{
                                log!("Funder is not council!");
                            }
                        }
                        _ => (),
                    };
            } else {
                env::panic_str("ERR_WRONG_VAL_RECEIVED")
            }
        },
        PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
        } 
    }
    
    #[private]
    pub fn callback_new_proposal(&mut self, dao_contract: String) -> Promise{
        // Get proposal ID from add_proposal promise
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!();
            },
            PromiseResult::Successful(val) => {
                if let Ok(proposal_id) = near_sdk::serde_json::from_slice::<u64>(&val) {
                    // ensure only DAO bot can call this method
                    require!(env::predecessor_account_id() == env::current_account_id(), "ONLY DAO BOT MAY CALL THIS METHOD");
                    
                    // Approve proposal that was just added 
                    ext_dao::ext(AccountId::try_from(dao_contract.clone().to_string()).unwrap())
                   .act_proposal(proposal_id, Action::VoteApprove, Some("Keypom DAO-Bot auto-registration".to_string()))
                } else {
                    env::panic_str("ERR_WRONG_VAL_RECEIVED")
                }
            },
            PromiseResult::Failed => {
                env::panic_str("ERR_CALL_FAILED");
            }
        } 
    }

    #[private]
    pub fn change_keypom_contract(&mut self, new_contract: String){
        self.keypom_contract = new_contract
    }

    pub fn view_keypom_contract(&self) -> String{
        self.keypom_contract.clone()
    }
}
