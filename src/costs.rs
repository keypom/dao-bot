use crate::*;

#[near_bindgen]
impl Marketplace {
    pub(crate) fn charge_deposit(&mut self, required_deposit: Balance) {
        let predecessor = env::predecessor_account_id();
        near_sdk::log!("Required cost: {}", required_deposit);
        require!(env::attached_deposit() >= required_deposit, "Insufficient Attached Deposit");

        let amount_to_refund = env::attached_deposit() - required_deposit;

        near_sdk::log!("Refunding {} excess deposit", amount_to_refund);
        Promise::new(predecessor).transfer(amount_to_refund);
        return;
    
    }
}