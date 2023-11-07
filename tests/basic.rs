use std::collections::HashMap;

use access_key_marketplace::{DropId, AddedDropDetails, ExtKeyData};
use near_sdk::{serde_json::json, PublicKey};
use near_workspaces;
use tokio;
use anyhow;
use near_units;

const KEYPOM_WASM_PATH: &str = "./__tests__/ext_wasm/keypom.wasm";
const MARKETPLACE_WASM_PATH: &str = "./out/access_key_marketplace.wasm";
const LINKDROP_WASM_PATH: &str = "./__tests__/ext_wasm/linkdrop.wasm";
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup and init both contracts
    let worker = near_workspaces::sandbox().await?;
    let linkdrop_wasm = std::fs::read(LINKDROP_WASM_PATH)?;
    let root = worker.dev_deploy(&linkdrop_wasm).await?;
    let keypom_wasm = std::fs::read(KEYPOM_WASM_PATH)?;
    let keypom = worker.dev_deploy(&keypom_wasm).await?;
    let marketplace_wasm = std::fs::read(MARKETPLACE_WASM_PATH)?;
    let marketplace = worker.dev_deploy(&marketplace_wasm).await?;
    let ali = worker.dev_create_account().await?;
    let bob = worker.dev_create_account().await?;

    // Init
    keypom.call("new").args_json(json!({
        "root_account": root.id(),
        "owner_id": keypom.id(),
        "contract_metadata": {
            "version": "3.0.0",
            "link": "foo"
        }
    }))
    .transact()
    .await?;

    marketplace.call("new").args_json(json!({
        "keypom_contract": keypom.id(),
        "contract_owner": marketplace.id()
    }))
    .transact()
    .await?;
    let deposit = near_units::parse_near!("1");
    
    // List Event
    let outcome = ali.call(marketplace.id(), "list_event").args_json(json!({
        "event_id": "moon-party",
        "max_markup": 2
    }))
    .deposit(deposit)
    .transact()
    .await?;
    assert!(outcome.is_success());

    // Creating drop
    outcome = ali.call(keypom.id(), "create_drop").args_json(json!({
        "drop_id": "drop-id-premium",
        "asset_data": [{
            "assets": [null],
            "uses": 2,
        }],
        "key_data": []
    }))
    .deposit(deposit)
    .transact()
    .await?;
    assert!(outcome.is_success());

    outcome = ali.call(keypom.id(), "create_drop").args_json(json!({
        "drop_id": "drop-id-normal",
        "asset_data": [{
            "assets": [null],
            "uses": 2,
        }],
        "key_data": []
    }))
    .deposit(deposit)
    .transact()
    .await?;
    assert!(outcome.is_success());

    let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
    .args_json(json!({
        "event_id": "moon-party"
    }))
    .await?.json()?;

    println!("--------------\n{}", result);

    // Add drops to event
    let added_drops: HashMap<DropId, AddedDropDetails> = HashMap::new();
    added_drops.insert(&"drop-id-normal", AddedDropDetails { max_tickets: 50, price_by_drop_id: 1 });
    added_drops.insert(&"drop-id-premium", AddedDropDetails { max_tickets: 5, price_by_drop_id: 3 });
    outcome = ali.call(marketplace.id(), "add_drop_to_event").args_json(json!({
        "event_id": "moon-party",
        "added_drops": added_drops
    }))
    .deposit(deposit)
    .transact()
    .await?;

    let result: serde_json::Value = worker.view(marketplace.id(), "get_event_information")
    .args_json(json!({
        "event_id": "moon-party"
    }))
    .await?.json()?;

    println!("--------------\n{}", result);


    // Attempting to buy without allowlist
    let key: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap();
    let new_key_info = ExtKeyData {
        public_key: key,
        key_owner: ali.id(),
        password_by_use: None,
        metadata: None

    };
    outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
        "event_id": "moon-party",
        "new_key_info": new_key_info,
        "ticket_tier": 2
    }))
    .deposit(deposit * 2)
    .transact()
    .await?;
    // Should not have enough money, allowlist
    assert!(outcome.is_failure());

    outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
        "event_id": "moon-party",
        "new_key_info": new_key_info,
        "ticket_tier": 1
    }))
    .deposit(deposit * 2)
    .transact()
    .await?;
    // should not work, allowlist
    assert!(outcome.is_failure());

    // Allow marketplace to add tickets
    outcome = ali.call(keypom.id(), "add_to_sale_allowlist").args_json(json!({
        "drop_id": "drop-id-normal",
        "account_ids": [marketplace.id()]
    }))
    .deposit(deposit)
    .transact()
    .await?;
    assert!(outcome.is_success());

    outcome = ali.call(keypom.id(), "add_to_sale_allowlist").args_json(json!({
        "drop_id": "drop-id-premium",
        "account_ids": [marketplace.id()]
    }))
    .deposit(deposit)
    .transact()
    .await?;
    assert!(outcome.is_success());

    // Should now fail based on balance, first should pass, second should not
    outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
        "event_id": "moon-party",
        "new_key_info": new_key_info,
        "ticket_tier": 1
    }))
    .deposit(deposit * 2)
    .transact()
    .await?;
    // should not work, allowlist
    assert!(outcome.is_success());

    outcome = ali.call(marketplace.id(), "buy_initial_sale").args_json(json!({
        "event_id": "moon-party",
        "new_key_info": new_key_info,
        "ticket_tier": 2
    }))
    .deposit(deposit * 2)
    .transact()
    .await?;
    // should not work, allowlist
    assert!(outcome.is_failure());

    Ok(())
    
}