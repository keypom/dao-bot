use crate::*;

// 0.1 $NEAR
pub const SPUTNIK_PROPOSAL_DEPOSIT: Balance = 100000000000000000000000;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    /// Modify a Drop's Resale Conditions
    #[payable]
    pub fn modify_drop_resale_markup(
        &mut self,
        event_id: EventID,
        new_markup: u64
    ){
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        event.max_markup = new_markup;
        self.event_by_id.insert(&event_id, &event);
    }
    
    // Modify a Key's Resale Conditions
    
    // Rovoke a Resale
    pub fn revoke_resale(
        &mut self,
        pubic_key: PublicKey,

    ){

    }
}