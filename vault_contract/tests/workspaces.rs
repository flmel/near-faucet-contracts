use workspaces::{Contract, DevNetwork, Worker};
async fn init(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    let wasm = std::fs::read("./target/wasm32-unknown-unknown/release/vault_contract.wasm")?;
    let contract = worker.dev_deploy(&wasm).await?;

    return Ok(contract);
}

#[tokio::test]
async fn add_to_allow_list() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = init(&worker).await?;

    let alice = worker.dev_create_account().await?;

    // Contract is the signer for the transaction
    let contract_res = contract
        .call("add_to_allow_list")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    // Alice is the signer for the transaction
    let alice_res = alice
        .call(contract.id(), "add_to_allow_list")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    assert!(contract_res.is_success());
    assert!(alice_res.is_failure());

    Ok(())
}
