mod ext_traits;
mod types;
mod models;

use near_sdk::collections::{LookupMap, UnorderedSet};
use types::*;
use models::*;
use ext_traits::{ext_keypom};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId, Gas, env, Promise, PromiseResult, require, Balance};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::PublicKey;
use std::convert::TryFrom;
use std::collections::{HashSet, HashMap};
use near_sdk::json_types::{U128, Base64VecU8};

pub const XCC_GAS: Gas = Gas(20_000_000_000_000);
pub const TGAS: u64 = 1_000_000_000_000;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    #[private]
    pub fn change_keypom_contract(&mut self, new_contract: AccountId){
        self.keypom_contract = new_contract
    }

    pub fn view_keypom_contract(&self) -> AccountId{
        self.keypom_contract.clone()
    }
}