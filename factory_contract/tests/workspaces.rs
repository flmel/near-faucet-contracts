use near_sdk::serde_json::json;
use workspaces::{
    network::Sandbox,
    types::{KeyType, SecretKey},
    Account, Contract, Worker,
};

async fn init() -> anyhow::Result<(Contract, Account, Worker<Sandbox>)> {
    let worker = workspaces::sandbox().await?;
    let factory_wasm = std::fs::read("res/factory_contract.wasm")?;

    let contract = worker.dev_deploy(&factory_wasm).await?;

    let account = worker.dev_create_account().await?;

    Ok((contract, account, worker))
}

#[tokio::test]
async fn create_contract() -> anyhow::Result<()> {
    let (factory_contract, account, worker) = init().await?;

    let res = factory_contract
        .call("create_contract")
        .args_json(json!({"desired_prefix": "token", "owner_id": account.id(), "total_supply": "100000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}))
        .gas(near_units::parse_gas!("300 T") as u64)
        .transact()
        .await?;

    println!("new_default_meta outcome: {:#?}", res);

    Ok(())
}
