use near_sdk::serde_json::json;
use workspaces::{Account, Contract};

async fn init() -> anyhow::Result<(Contract, Account)> {
    let worker = workspaces::sandbox().await?;
    let factory_wasm = std::fs::read("res/factory_contract.wasm")?;

    let contract = worker.dev_deploy(&factory_wasm).await?;
    let account = worker.dev_create_account().await?;

    Ok((contract, account))
}

#[tokio::test]
async fn create_contract() -> anyhow::Result<()> {
    let (factory_contract, account) = init().await?;

    // create a new ft contract with metadata on a subaccount of the factory contract
    let create_contract_res = factory_contract
        .call("create_contract")
        .args_json(json!({"desired_prefix": "token", "owner_id": account.id(), "total_supply": "100000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}))
        .max_gas()
        .transact()
        .await?;

    let num_contracts_res = factory_contract
        .view("num_contracts")
        .await?
        .json::<u64>()?;

    // deploy ft contract on the subaccount
    assert!(create_contract_res.is_success());
    // we have ft one contract deployed
    assert_eq!(num_contracts_res, 1);

    Ok(())
}

#[tokio::test]
async fn delete_contract_account() -> anyhow::Result<()> {
    let (factory_contract, account) = init().await?;

    // create a new ft contract with metadata on a subaccount of the factory contract
    let _create_contract_res = factory_contract
        .call("create_contract")
        .args_json(json!({"desired_prefix": "token", "owner_id": account.id(), "total_supply": "100000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}))
        .max_gas()
        .transact()
        .await?;

    let delete_contract_res = factory_contract
        .call("delete_contract_account")
        .args_json(json!({"ft_contract_id": "token.".to_owned() + factory_contract.id()}))
        .max_gas()
        .transact()
        .await?;

    assert!(delete_contract_res.is_success());

    Ok(())
}

#[tokio::test]
async fn num_contracts() -> anyhow::Result<()> {
    let (factory_contract, account) = init().await?;

    // create first ft contract
    let create_contract2_res = factory_contract
        .call("create_contract")
        .args_json(json!({"desired_prefix": "token", "owner_id": account.id(), "total_supply": "100000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}))
        .max_gas()
        .transact()
        .await?;

    // create second ft contract
    let create_contract_res2 = factory_contract
        .call("create_contract")
        .args_json(json!({"desired_prefix": "token2", "owner_id": account.id(), "total_supply": "100000000000", "metadata": { "spec": "ft-1.0.0", "name": "Example Token Name", "symbol": "EXLT", "decimals": 8 }}))
        .max_gas()
        .transact()
        .await?;

    let num_contracts_res = factory_contract
        .view("num_contracts")
        .await?
        .json::<u64>()?;

    assert!(create_contract2_res.is_success());
    assert!(create_contract_res2.is_success());
    assert_eq!(num_contracts_res, 2);

    let delete_contract_res = factory_contract
        .call("delete_contract_account")
        .args_json(json!({"ft_contract_id": "token.".to_owned() + factory_contract.id()}))
        .max_gas()
        .transact()
        .await?;

    let num_contracts_res2 = factory_contract
        .view("num_contracts")
        .await?
        .json::<u64>()?;

    assert!(delete_contract_res.is_success());

    assert_eq!(num_contracts_res2, 1);

    Ok(())
}
