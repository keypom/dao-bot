use near_sdk::ext_contract;

use crate::*;

#[ext_contract(ext_dao)]
trait ExtDao{

    fn get_policy(&self) -> Policy;

    fn act_proposal(&mut self, id: u64, action: Action, memo: Option<String>);

    fn add_proposal(&mut self, proposal: ProposalInput);
}

// #[ext_contract(ext_self)]
// trait ContractExt{
//     fn get_roles_callback(&self);
// }

