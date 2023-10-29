use crate::*;

#[near_bindgen]
impl Marketplace {
    /// Set the contract to be frozen thus not allowing any drops to be created or keys added
    pub fn freeze_contract(&mut self) {
        self.assert_owner();
        self.global_freeze = true
    }
    
    /// Set the contract to be unfrozen thus resuming the ability for drops and keys to be created
    pub fn unfreeze_contract(&mut self) {
        self.assert_owner();
        self.global_freeze = false;
    }
    
    /// Helper method to check if the predecessor is the current contract owner
    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.contract_owner_id,
            "Only the contract owner can call this function"
        );
    }
}
