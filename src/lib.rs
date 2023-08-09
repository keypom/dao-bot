mod ext_traits;

use ext_traits::{ext_dao, ext_sbt_registry};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId, Gas, env, Promise, PromiseResult, require, Balance};
use near_sdk::serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::collections::HashSet;
use near_sdk::json_types::{U128, Base64VecU8};

pub const XCC_GAS: Gas = Gas(20_000_000_000_000);
pub const TGAS: u64 = 1_000_000_000_000;

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

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct OwnedToken {
    pub token: TokenId,
    pub metadata: TokenMetadata,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub class: ClassId,                      // token class
    pub issued_at: Option<u64>, // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<u64>, // When token expires, Unix epoch in milliseconds
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

pub type ClassId = u64;
pub type TokenId = u64;

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
    keypom_contract: AccountId
}

impl Default for Contract{
    fn default() -> Self{
        Self{
            keypom_contract: AccountId::try_from("v2.keypom.near".to_string()).unwrap()
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {

    #[payable]
    pub fn new_auto_registration(&mut self, dao_contract: AccountId, keypom_args: KeypomArgs, funder: AccountId, proposal: ProposalInput, human_only: Option<bool>) {
        // Ensure Keypom called this function 
        require!(env::predecessor_account_id() == self.keypom_contract.clone(), "KEYPOM MUST BE PREDECESSOR, CHECK REQUIRED VERSION USING view_keypom_contract");
        
        // Note since ONLY AddMemberToRole defined from proposal.kind, any other proposal types will result in serialization error!
        require!(keypom_args.funder_id_field == Some("funder".to_string()) && keypom_args.account_id_field == Some("proposal.kind.AddMemberToRole.member_id".to_string()), "KEYPOM MUST SEND THESE ARGS");

        // Ensure enough attached deposit was added to add the proposal
        require!(env::attached_deposit() >= SPUTNIK_PROPOSAL_DEPOSIT, "ATTACH MORE NEAR, AT LEAST 0.1 $NEAR");

        // Ensure proposal kind is valid
        match &proposal.kind{
            // extract member ID here using match statement
            ProposalKind::AddMemberToRole { member_id, role: _ } => {
                // If Proof-of-Humanity required, begin check
                if human_only.unwrap_or(false) {
                    ext_sbt_registry::ext(AccountId::try_from("registry.i-am-human.near".to_string()).unwrap())
                       .is_human(member_id.clone())
                       .then(
                            Self::ext(env::current_account_id())
                            .internal_human_check(funder, proposal, dao_contract)
                        );
                }
                // If no humanity proof required, start check right away.
                else{
                    // Begin auto-registration
                    ext_dao::ext(AccountId::try_from(dao_contract.clone().to_string()).unwrap())
                    .get_policy()
                    .then(
                        Self::ext(env::current_account_id())
                        .internal_get_roles_callback(funder, proposal, dao_contract)
                    );
                }
            }
        }
    } 

    #[private]
    pub fn internal_human_check(funder: AccountId, proposal: ProposalInput, dao_contract: AccountId) {
         // Parse Response and Check if Fractal is in owned tokens
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            if let Ok(proof) = near_sdk::serde_json::from_slice::<Vec<(AccountId, Vec<ClassId>)>>(&val) {
                let mut human_tokens = proof.into_iter().peekable();
                log!("New Human Check");
                require!(human_tokens.peek().is_none() == false, "CLAIMING ACCOUNT MUST BE HUMAN");
                
                // Begin auto-registration
                ext_dao::ext(AccountId::try_from(dao_contract.clone().to_string()).unwrap())
                .get_policy()
                .then(
                    Self::ext(env::current_account_id())
                    .internal_get_roles_callback(funder, proposal, dao_contract)
                );
            } else {
             env::panic_str("ERR_WRONG_VAL_RECEIVED")
            }      
        }
        else{
            env::panic_str("PROBLEM WITH PROMISE")
        }  
    }

    
    // Roles callback, parse and return council role(s)
    #[private]
    pub fn internal_get_roles_callback(&mut self, funder: AccountId, proposal: ProposalInput, dao_contract: AccountId){
        // Receive get_policy promise, parse it and see if funder is on DAO council
        if let PromiseResult::Successful(val) = env::promise_result(0) {
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
                                    .callback_new_auto_registration(dao_contract)
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
        } 
    }
    
    #[private]
    pub fn callback_new_auto_registration(&mut self, dao_contract: AccountId) -> Promise{
        // Get proposal ID from add_proposal promise
        if let PromiseResult::Successful(val) = env::promise_result(0) {
            if let Ok(proposal_id) = near_sdk::serde_json::from_slice::<u64>(&val) {                 
                // Approve proposal that was just added 
                ext_dao::ext(AccountId::try_from(dao_contract.clone().to_string()).unwrap())
               .act_proposal(proposal_id, Action::VoteApprove, Some("Keypom DAO BOT Auto-Registration".to_string()))
            } else {
                env::panic_str("ERR_WRONG_VAL_RECEIVED")
            }
        } 
        else{
            env::panic_str("PROBLEM WITH PROMISE")
        }  
    }

    #[private]
    pub fn change_keypom_contract(&mut self, new_contract: AccountId){
        self.keypom_contract = new_contract
    }

    pub fn view_keypom_contract(&self) -> AccountId{
        self.keypom_contract.clone()
    }
}
