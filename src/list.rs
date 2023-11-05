use crate::*;

// Implement the contract structure
#[near_bindgen]
impl Marketplace {

    // *********** ASSUMING ALL NEW DROPS WITH NO KEYS ***********

    /// List an event
    #[payable]
    pub fn list_event(
        &mut self, 
        event_id: EventID,
        event_name: Option<String>,
        description: Option<String>,
        date: Option<String>,
        host: Option<AccountId>,
        max_markup: u64,
        max_tickets: Option<HashMap<DropId, Option<u64>>>,
        drop_ids: Option<Vec<DropId>>,
        price_by_drop_id: Option<HashMap<DropId, Option<Balance>>>,
        // Implement this later, not high priority right now and could be complicated
        //existing_keys: Option<HashMap<DropId, Vec<PublicKey>>>
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        let final_event_details = self.create_event_details(
            event_id, 
            event_name, 
            description, 
            date, 
            host, 
            max_markup, 
            max_tickets, 
            drop_ids, 
            price_by_drop_id);

        // Insert by event ID stuff first
        self.event_by_id.insert(&final_event_details.event_id, &final_event_details);
        self.resales_for_event.insert(&final_event_details.event_id, &None);

        // By Drop ID data structures
        let drop_ids = final_event_details.drop_ids;
        for drop_id in drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &final_event_details.event_id);
            // if let Some(pub_key) = &existing_keys.as_ref().unwrap().get(&drop_id){
            //     self.keys_by_drop_id.insert(&drop_id, &Some(pub_key.to_vec()));
            // }
            self.listed_keys_per_drop.insert(&drop_id, &None);
        }

        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(storage_cost);
    }
    
    // List a ticket, apply constraints from drop or generate own if not associated with known drop
    #[payable]
    pub fn list_ticket(
        &mut self,
        key: ExtKeyData,
        price: Balance,
        approval_id: u64,
    ){
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        near_sdk::log!("initial bytes {}", initial_storage);

        // Predecessor must either own the key, or sign the txn using the key!
        require!(env::predecessor_account_id() == key.key_owner.clone().unwrap_or(env::current_account_id()) 
        || env::signer_account_pk() == key.public_key, "Must own or use the access key being listed!");
        
        // Get key's drop ID and then event, in order to modify all needed data
        ext_keypom::ext(AccountId::try_from(self.keypom_contract.to_string()).unwrap())
                       .get_key_information(String::try_from(&key.public_key).unwrap())
                       .then(
                            Self::ext(env::current_account_id())
                            .internal_list_ticket(key, price, approval_id, initial_storage)
                        );
    }

    #[private]
    pub fn internal_list_ticket(
        &mut self,
        key: ExtKeyData,
        price: Balance,
        approval_id: u64,
        initial_storage: u64
    ){
        
         // Parse Response and Check if Fractal is in owned tokens
         if let PromiseResult::Successful(val) = env::promise_result(0) {
            // expected result: Result<ExtKeyInfo, String>
            
            if let Ok(key_info) = near_sdk::serde_json::from_slice::<Result<ExtKeyInfo, String>>(&val) {
                let drop_id = key_info.unwrap().drop_id;
                
                // Case 1: Key associated with event
                // Data structures to update: event_by_id, resales_for_event, listed_keys_per_drop, approval_id_by_pk, resale_per_pk
                if let Some(event_id) = self.event_by_drop_id.get(&drop_id).as_ref(){
                    let event = self.event_by_id.get(&event_id).expect("No event found for Event ID");
                    
                    // Clamp price using max_markup
                    let mut final_price = price;
                    if let Some(base_price) = event.price_by_drop_id.get(&drop_id){
                        if base_price.is_some() && price.gt(&(base_price.unwrap() * event.max_markup as u128)){
                            let max_price = base_price.unwrap() * event.max_markup as u128;
                            final_price = base_price.unwrap();
                            if price.gt(&max_price){
                                final_price = max_price;
                            }
                        }
                    }

                    // Resale per PK and approval ID per PK
                    self.resale_per_pk.insert(&key.public_key, &final_price);
                    self.approval_id_by_pk.insert(&key.public_key, &approval_id);
                    
                    // ensure listed keys per drop contains this key
                    if self.listed_keys_per_drop.contains_key(&drop_id){
                        if self.listed_keys_per_drop.get(&drop_id).is_none(){
                            // No existing vector
                            let mut keys_vec: Vec<PublicKey> = Vec::new();
                            keys_vec.push(key.public_key.clone());
                            self.listed_keys_per_drop.insert(&drop_id, &Some(keys_vec));
                        }else{
                            self.listed_keys_per_drop.get(&drop_id).unwrap().unwrap().push(key.public_key.clone());
                        }
                    }else{
                       // Create new drop <-> vector pairing
                       let mut keys_vec: Vec<PublicKey> = Vec::new();
                       keys_vec.push(key.public_key.clone());
                       self.listed_keys_per_drop.insert(&drop_id, &Some(keys_vec));
                    }

                    // ensure resales for event contains this key
                    if self.resales_for_event.contains_key(&event_id){
                        if self.resales_for_event.get(&event_id).is_none(){
                            // No existing vector
                            let mut keys_vec: Vec<PublicKey> = Vec::new();
                            keys_vec.push(key.public_key.clone());
                            self.resales_for_event.insert(&event_id, &Some(keys_vec));
                        }else{
                            self.resales_for_event.get(&event_id).unwrap().unwrap().push(key.public_key.clone());
                        }
                    }else{
                       // Create new drop <-> vector pairing
                       let mut keys_vec: Vec<PublicKey> = Vec::new();
                       keys_vec.push(key.public_key.clone());
                       self.resales_for_event.insert(&event_id, &Some(keys_vec));
                    }
                }
                 // Case 2: Key not associated with event
                 // Data Structures to update: max_price_per_dropless_key, approval_id_by_pk, resale_per_pk, listed_keys_per_drop, approved_drops
                else{
                    // Add to dropless key data structures - max_price, approval by pk, resale by pk, approved drops, listed keys per drop
                    let max_price = price * 2;
                    self.max_price_per_dropless_key.insert(&key.public_key, &max_price);
                    self.approval_id_by_pk.insert(&key.public_key, &approval_id);
                    self.resale_per_pk.insert(&key.public_key, &price);
                    if !self.approved_drops.contains(&drop_id){
                        self.approved_drops.insert(drop_id.clone());
                    }
                    
                    if self.listed_keys_per_drop.contains_key(&drop_id){
                        if self.listed_keys_per_drop.get(&drop_id).is_none(){
                            // No existing vector
                            let mut keys_vec: Vec<PublicKey> = Vec::new();
                            keys_vec.push(key.public_key.clone());
                            self.listed_keys_per_drop.insert(&drop_id, &Some(keys_vec));
                        }else{
                            self.listed_keys_per_drop.get(&drop_id).unwrap().unwrap().push(key.public_key.clone());
                        }
                    }else{
                        // Create new drop <-> vector pairing
                        let mut keys_vec: Vec<PublicKey> = Vec::new();
                        keys_vec.push(key.public_key.clone());
                        self.listed_keys_per_drop.insert(&drop_id, &Some(keys_vec));
                    }

                }
            } else {
             env::panic_str("ERR_WRONG_VAL_RECEIVED")
            }      
        }
        else{
            env::panic_str("Invalid Key, not found on Keypom Contract!")
        }  
        
        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(storage_cost);
    }


    // TODO: VERIFY IF ALL NECESSARY DATA STRUCTURES ARE UPDATED HERE
    // Add drop to an existing event
    #[payable]
    pub fn add_drop_to_event(
        &mut self, 
        event_id: EventID,
        added_drops: HashMap<DropId, AddedDropDetails>,
        // Implement this later, not high priority right now and could be complicated
        //existing_keys: Option<HashMap<DropId, Vec<PublicKey>>>
    ){
        // Data Structures to update: event_by_id (EventDetails), approved_drops, event_by_drop_id, listed_keys_per_drop
        // EventDetails fields to update: max_tickets, drop_ids, price_by_drop_ids

        // Ensure no global freeze and event exists
        self.assert_no_global_freeze();
        let initial_storage = env::storage_usage();
        require!(self.event_by_id.get(&event_id).is_some(), "Event not found!");
        

        // Update event details
        let mut event = self.event_by_id.get(&event_id).expect("No Event Found");
        let added_drops_vec = added_drops.iter();
        let mut drop_ids: Vec<String> = Vec::new();

        for (key, val) in added_drops_vec{
            event.drop_ids.push(key.to_string());
            event.max_tickets.insert(key.to_string(), val.max_tickets);
            event.price_by_drop_id.insert(key.to_string(), val.price_by_drop_id);
            drop_ids.push(key.to_string());
        }

        self.event_by_id.insert(&event_id, &event);

        for drop_id in drop_ids {
            self.approved_drops.insert(drop_id.clone());
            self.event_by_drop_id.insert(&drop_id, &event_id);
            // if let Some(pub_key) = &existing_keys.as_ref().unwrap().get(&drop_id){
            //     self.keys_by_drop_id.insert(&drop_id, &Some(pub_key.to_vec()));
            // }
            self.listed_keys_per_drop.insert(&drop_id, &None);
        }
        
        // Calculate used storage and charge the user
        let net_storage = env::storage_usage() - initial_storage;
        let storage_cost = net_storage as Balance * env::storage_byte_cost();

        self.charge_deposit(storage_cost);
    }
}