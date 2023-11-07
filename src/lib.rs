#![allow(unused_imports)]

pub mod models;
pub mod buy;
pub mod costs;
pub mod ext_traits;
pub mod helper;
pub mod list;
pub mod modify_sale;
pub mod owner;
pub mod types;
pub mod view;

pub use models::*;
pub use buy::*;
pub use costs::*;
pub use ext_traits::*;
pub use helper::*;
pub use list::*;
pub use modify_sale::*;
pub use owner::*;
pub use types::*;
pub use view::*;



use near_sdk::collections::{LookupMap, UnorderedSet, UnorderedMap};
use types::*;
use models::*;
use ext_traits::ext_keypom;
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

// TODO: VERIFY PUBLIC-KEY VS TOKEN_ID ON KEYPOM SIDE, WHAT IS NEEDED?

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Marketplace {
    /// **************** Admin Stuff ****************
    /// Owner of the contract that can set configurations such as global freezes etc.
    pub contract_owner_id: AccountId,
    /// Whether or not the contract is frozen and no new drops can be created / keys added.
    pub global_freeze: bool,
    
    /// **************** Keypom ****************
    pub keypom_contract: AccountId,

    // TODO: Change vars that need to be iterable to UnorderedMap or HashSet
    // **************** By Event ID ****************
    // Event/Drop Information per Drop
    pub event_by_id: UnorderedMap<EventID, EventDetails>,
    // Key resales per event
    // Frontend can get drop ID and thus base pricing for each key using get_key_information_batch 
    pub resales_for_event: LookupMap<EventID, Option<Vec<PublicKey>>>,

    // TODO: STORE KEY PASSWORD SOMEWHERE? HOW DOES FRONTEND KNOW WHAT PASSWORD TO PASS IN?

    // **************** By Drop ****************
    // Drops that the marketplace can add keys to, by DropID
    pub approved_drops: HashSet<DropId>,
    // Event ID given a drop ID
    pub event_by_drop_id: LookupMap<DropId, EventID>,
    // Collection of keys that have been listed per drop
    pub listed_keys_per_drop: LookupMap<DropId, Option<Vec<PublicKey>>>,

    // **************** By Key ****************
    // Collection of keys that have been listed per drop -> will use and implement IF ccc's are too much
    //pub drop_per_key: LookupMap<PublicKey, DropId>,
    // Price ceiling for listed keys not part of drop, generated automatically on first resale
    pub max_price_per_dropless_key: LookupMap<PublicKey, Balance>,
    // Approval ID by Public Key, used when user lists key for sale
    pub approval_id_by_pk: LookupMap<PublicKey, u64>,
    // Resale price per pk
    pub resale_per_pk: LookupMap<PublicKey, Balance>
}

impl Default for Marketplace{
    fn default() -> Self{
        Self{
            /// **************** Admin Stuff ****************
            contract_owner_id: AccountId::try_from("minqi.testnet".to_string()).unwrap(),
            global_freeze: false,
            /// **************** Keypom ****************
            keypom_contract: AccountId::try_from("testing-nearcon-keypom.testnet".to_string()).unwrap(),
            // **************** By Event ID ****************
            event_by_id: UnorderedMap::new(StorageKeys::EventInfoPerDrop),
            resales_for_event: LookupMap::new(StorageKeys::ResalePerEvent),
            // **************** By Drop ****************
            approved_drops: HashSet::new(),
            event_by_drop_id: LookupMap::new(StorageKeys::EventByDropId),
            listed_keys_per_drop: LookupMap::new(StorageKeys::KeysByDropId),
            // **************** By Key ****************
            //drop_per_key: LookupMap::new(StorageKeys::KeysPerDrop),
            max_price_per_dropless_key: LookupMap::new(StorageKeys::MaxPricePerKey),
            approval_id_by_pk: LookupMap::new(StorageKeys::ApprovalIDByPk),
            resale_per_pk: LookupMap::new(StorageKeys::ResaleForPK),
        }
    }
}



#[near_bindgen]
impl Marketplace {

    #[init]
    pub fn new(
        contract_owner: String,
        keypom_contract: String
    ) -> Self {
        Self {
             /// **************** Admin Stuff ****************
             contract_owner_id: AccountId::try_from(contract_owner.to_string()).unwrap(),
             global_freeze: false,
             /// **************** Keypom ****************
             keypom_contract: AccountId::try_from(keypom_contract.to_string()).unwrap(),
             // **************** By Event ID ****************
             event_by_id: UnorderedMap::new(StorageKeys::EventInfoPerDrop),
             resales_for_event: LookupMap::new(StorageKeys::ResalePerEvent),
             // **************** By Drop ****************
             approved_drops: HashSet::new(),
             event_by_drop_id: LookupMap::new(StorageKeys::EventByDropId),
             listed_keys_per_drop: LookupMap::new(StorageKeys::KeysByDropId),
             // **************** By Key ****************
             //drop_per_key: LookupMap::new(StorageKeys::KeysPerDrop),
             max_price_per_dropless_key: LookupMap::new(StorageKeys::MaxPricePerKey),
             approval_id_by_pk: LookupMap::new(StorageKeys::ApprovalIDByPk),
             resale_per_pk: LookupMap::new(StorageKeys::ResaleForPK),
        }
    }

    /// Helper function to make sure there isn't a global freeze on the contract
    pub(crate) fn assert_no_global_freeze(&self) {
        if env::predecessor_account_id() != self.contract_owner_id {
            require!(self.global_freeze == false, "Contract is frozen and no new drops or keys can be created");
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
