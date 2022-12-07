use near_units::parse_near;
use workspaces::{network::Sandbox, Account, Contract, Worker};

async fn init() -> anyhow::Result<(Contract, Account, Worker<Sandbox>)> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read("./target/wasm32-unknown-unknown/release/vault_contract.wasm")?;

    let contract = worker
        .root_account()
        .unwrap()
        .create_subaccount("contract")
        .initial_balance(parse_near!("30000 N"))
        .transact()
        .await?
        .into_result()?
        .deploy(&wasm)
        .await?
        .into_result()?;

    let alice = worker.dev_create_account().await?;

    return Ok((contract, alice, worker));
}

#[tokio::test]
async fn integration_request_funds() -> anyhow::Result<()> {
    let (contract, alice, _worker) = init().await?;
    // Add alice to whitelist
    _ = contract
        .call("add_to_whitelist")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    // Request funds from contract
    let alice_res = alice
        .call(&contract.id(), "request_funds")
        .transact()
        .await?;

    // Check that alice's balance increased
    assert!(alice_res.is_success());
    assert!(alice.view_account().await?.balance > parse_near!("10000 N"));

    // Request funds from contract without waiting for the 24 hour cooldown
    let second_alice_res = alice
        .call(&contract.id(), "request_funds")
        .transact()
        .await?;

    // Cooldown period has not passed, failure expected
    assert!(second_alice_res.is_failure());

    Ok(())
}

#[tokio::test]
async fn integration_add_to_whitelist() -> anyhow::Result<()> {
    let (contract, alice, _worker) = init().await?;

    // Contract is the signer for the transaction
    let contract_res = contract
        .call("add_to_whitelist")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    // Alice is the signer for the transaction
    let alice_res = alice
        .call(contract.id(), "add_to_whitelist")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    assert!(contract_res.is_success());
    // fn is #[private]
    assert!(alice_res.is_failure());

    Ok(())
}

#[tokio::test]
async fn integration_remove_from_whitelist() -> anyhow::Result<()> {
    let (contract, alice, _worker) = init().await?;
    // Contract is the signer for the transaction
    let contract_res = contract
        .call("remove_from_whitelist")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    // Alice is the signer for the transaction
    let alice_res = alice
        .call(contract.id(), "remove_from_whitelist")
        .args_json((&alice.id(),))
        .transact()
        .await?;

    assert!(contract_res.is_success());
    // fn is #[private]
    assert!(alice_res.is_failure());

    Ok(())
}
