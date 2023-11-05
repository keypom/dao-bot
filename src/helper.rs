use crate::*;

#[near_bindgen]
impl Marketplace{
    pub(crate) fn create_event_details(
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
    ) -> EventDetails{

        require!(self.event_by_id.get(&event_id).is_none(), "Event ID already exists!");

        let event_details = EventDetails{
            name: event_name,
            host: Some(host.unwrap_or(env::predecessor_account_id())),
            event_id,
            status: Status::Active,
            description,
            date,
            max_markup,
            max_tickets: max_tickets.unwrap_or(HashMap::new()),
            drop_ids: drop_ids.unwrap_or(Vec::new()),
            price_by_drop_id: price_by_drop_id.unwrap_or(HashMap::new())

        };
        return event_details
    }
}