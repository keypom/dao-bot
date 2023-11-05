use near_sdk::serde_json::json;
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
    
    let outcome1 = ali.call(marketplace.id(), "list_event").args_json(json!({
        "event_id": "moon-party",
        "max_markup": 2
    }))
    .deposit(deposit)
    .transact()
    .await?;

    println!("list_event outcome: {:#?}", outcome1);

    let outcome2 = ali.call(keypom.id(), "create_drop").args_json(json!({
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

    println!("premium drop creation outcome: {:#?}", outcome2);

    let outcome3 = ali.call(keypom.id(), "create_drop").args_json(json!({
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

    println!("normal drop creation outcome: {:#?}", outcome3);

   

    let result: serde_json::Value = worker.view(keypom.id(), "get_event_information")
    .args_json(json!({
        "event_id": "moon-party"
    }))
    .await?.json()?;

    println!("--------------\n{}", result);

    Ok(())
    
}




