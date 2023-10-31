use near_sdk::BorshStorageKey;

use crate::*;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    //AssetById { drop_id_hash: CryptoHash },
    //TokensPerOwnerInner { account_id_hash: CryptoHash },
    ResalePerEvent,
    EventInfoPerDrop,
    
    EventByDropId,
    KeysByDropId,
    
    MaxPricePerKey,
    ApprovalIDByPk,
    ResaleForPK,
}