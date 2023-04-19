/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

mod ext_traits;

use ext_traits::ext_dao;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId, Gas, env, Promise, PromiseResult};
use near_sdk::serde::{Deserialize, Serialize};
use std::convert::{TryFrom};
use std::collections::HashSet;
use near_sdk::json_types::U128;


// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
pub const XCC_GAS: Gas = Gas(20_000_000_000_000);

#[derive(Serialize, Deserialize)]
pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

#[derive(Serialize, Deserialize)]
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
pub struct DaoBot {
    message: String,
}

// Define the default, which automatically initializes the contract
impl Default for DaoBot{
    fn default() -> Self{
        Self{message: DEFAULT_MESSAGE.to_string()}
    }
}

// Implement the contract structure
#[near_bindgen]
impl DaoBot {
    // Public method - returns the greeting saved, defaulting to DEFAULT_MESSAGE
    pub fn new_proposal(proposal: ProposalInput, keypom_args: KeypomArgs, funder: String, member: String) {
        
    }

    pub fn get_roles(dao_contract: String) {
        ext_dao::ext(AccountId::try_from(dao_contract.to_string()).unwrap())
        .get_policy()
        .then(
            Self::ext(env::current_account_id())
            .get_roles_callback()
        );
    }

    pub fn get_roles_callback(){

    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn callback_new_proposal() {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(is_allowlisted) = near_sdk::serde_json::from_slice::<bool>(&val) {
                    is_allowlisted
                } else {
                    env::panic_str("ERR_WRONG_VAL_RECEIVED")
                }
            },
            PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
    }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_greeting() {
        
    }

    #[test]
    fn set_then_get_greeting() {
        
    }
}
