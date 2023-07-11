use near_sdk::ext_contract;

use crate::*;

#[ext_contract(ext_dao)]
trait ExtDao{

    fn get_policy(&self) -> Policy;

    fn act_proposal(&mut self, id: u64, action: Action, memo: Option<String>);

    fn add_proposal(&mut self, proposal: ProposalInput);
}

#[ext_contract(ext_sbt_registry)]
trait ExtSBTRegistry{

    fn is_human(&self, account: AccountId) -> Vec<(AccountId, Vec<ClassId>)>;

    fn sbt_tokens_by_owner(&self, account: AccountId, issuer: Option<AccountId>, from_class: Option<u64>, limit: Option<u32>, with_expired: Option<bool>) -> Vec<(AccountId, Vec<OwnedToken>)>;
}

// #[ext_contract(ext_self)]
// trait ContractExt{
//     fn get_roles_callback(&self);
// }


