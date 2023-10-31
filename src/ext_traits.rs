
use near_sdk::ext_contract;


use crate::*;


#[ext_contract(ext_keypom)]
trait ExtKeypom{

    fn add_keys(&mut self, drop_id: DropId, key_data: Vec<ExtKeyData>, keep_excess_deposit: Option<bool>) -> bool;

    fn get_key_information(&self, key: String) -> Result<ExtKeyInfo, String>;

    fn nft_token(&self, token_id: TokenId) -> Option<ExtNFTKey>;

    fn nft_transfer(&mut self, token_id: Option<TokenId>, receiver_id: Option<AccountId>, approval_id: Option<u64>, memo: PublicKey);

}

// #[ext_contract(ext_self)]
// trait ContractExt{
//     fn get_roles_callback(&self);
// }


