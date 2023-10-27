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

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Marketplace {
    /// Keypom
    keypom_contract: AccountId,

    // **************** By Drop ****************
    // Drops that the marketplace can add keys to, by DropID
    pub approved_drops: HashSet<AccountId>,
    // Resale Conditions per Drop, including max price
    pub drop_resale_conditions: LookupMap<DropId, ResaleConditions>,

    // **************** By Key ****************
    // Collection of keys that have been listed per drop
    pub listed_keys_per_drop: LookupMap<DropId, UnorderedSet<PublicKey>>,
    // Price ceiling for listed keys not part of drop, generated automatically on first resale
    pub max_price_per_dropless_key: LookupMap<PublicKey, Balance>,
    // Approval ID by Public Key, used when user lists key for sale
    pub approval_id_by_pk: LookupMap<PublicKey, u64>,


}

impl Default for Marketplace{
    fn default() -> Self{
        Self{
            keypom_contract: AccountId::try_from("v2.keypom.near".to_string()).unwrap(),
            approved_drops: HashSet::new(),
            drop_resale_conditions: LookupMap::new(StorageKeys::ResalePerDrop),
            listed_keys_per_drop: LookupMap::new(StorageKeys::KeysPerDrop),
            max_price_per_dropless_key: LookupMap::new(StorageKeys::MaxPricePerKey),
            approval_id_by_pk: LookupMap::new(StorageKeys::ApprovalIDByPk)
        }
    }
}

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
