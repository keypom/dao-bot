use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    // *********** ASSUMING ALL NEW DROPS WITH NO KEYS ***********

    /// List an event
    #[payable]
    pub fn list_event(
        &mut self, 
        event_details: EventDetails,
        resale_conditions: ResaleConditions,
        // Implement this later, not high priority right now and could be complicated
        //existing_keys: Option<HashMap<DropId, Vec<PublicKey>>>
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        let mut final_event_details = event_details.clone();
        
        // Ensure no event ID collisions.
        // Note that Frontend should pass in event ID with date.now() to minimize this
        let mut new_event_id = final_event_details.event_id;
        if self.event_info_by_id.contains_key(&event_details.event_id) {
            new_event_id.push_str(&env::predecessor_account_id().to_string());
            final_event_details.event_id = new_event_id.clone();
        }

        // Insert by event ID stuff first
        self.event_info_by_id.insert(&new_event_id, &event_details);
        self.event_resale_conditions.insert(&new_event_id, &resale_conditions);

        // By Drop ID data structures
        let drop_ids = final_event_details.drop_ids;
        for drop_id in drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &new_event_id);
            // if let Some(pub_key) = &existing_keys.as_ref().unwrap().get(&drop_id){
            //     self.keys_by_drop_id.insert(&drop_id, &Some(pub_key.to_vec()));
            // }
            self.keys_by_drop_id.insert(&drop_id, &None);
        }

        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(storage_cost);
    }
    
    // List a ticket, apply constraints from drop or generate own if not associated with known drop

    // Add drop to an existing event
    pub fn add_drop_to_event(
        &mut self, 
        event_id: EventID,
        drop_ids: Vec<DropId>,
        // Implement this later, not high priority right now and could be complicated
        //existing_keys: Option<HashMap<DropId, Vec<PublicKey>>>
    ){
        // Ensure no global freeze and event exists
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        require!(self.event_info_by_id.contains_key(&event_id) && self.event_resale_conditions.contains_key(&event_id), "Event not found!");

        for drop_id in drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &event_id);
            // if let Some(pub_key) = &existing_keys.as_ref().unwrap().get(&drop_id){
            //     self.keys_by_drop_id.insert(&drop_id, &Some(pub_key.to_vec()));
            // }
            self.keys_by_drop_id.insert(&drop_id, &None);
        }
        
        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(storage_cost);
    }
}