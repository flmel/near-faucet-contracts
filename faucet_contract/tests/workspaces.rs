use near_sdk::json_types::U128;
use near_units::parse_near;
use workspaces::{network::Sandbox, Account, Contract, Worker};

async fn init() -> anyhow::Result<(Contract, Account, Worker<Sandbox>)> {
    let worker = workspaces::sandbox().await?;
    let faucet_wasm =
        std::fs::read("./target/wasm32-unknown-unknown/release/near_testnet_faucet.wasm")?;

    // faucet contract with 5500 near
    let faucet_contract = worker
        .root_account()
        .unwrap()
        .create_subaccount("contract")
        .initial_balance(parse_near!("5500 N"))
        .transact()
        .await?
        .into_result()?
        .deploy(&faucet_wasm)
        .await?
        .into_result()?;

    // create alice account
    let alice = worker.dev_create_account().await?;

    return Ok((faucet_contract, alice, worker));
}

async fn create_vault_contract() -> anyhow::Result<Contract> {
    let vault_wasm = std::fs::read(
        "../vault_contract/target/wasm32-unknown-unknown/release/vault_contract.wasm",
    )?;

    let worker = workspaces::sandbox().await?;

    let vault_contract = worker
        .root_account()
        .unwrap()
        .create_subaccount("vault_contract")
        .initial_balance(parse_near!("50000 N"))
        .transact()
        .await?
        .into_result()?
        .deploy(&vault_wasm)
        .await?
        .into_result()?;

    return Ok(vault_contract);
}

#[tokio::test]
async fn test_request_near() -> anyhow::Result<()> {
    let (contract, alice, worker) = init().await?;
    let bob = worker.dev_create_account().await?;

    let initial_faucet_balance = contract.view_account().await?.balance;
    let initial_alice_balance = alice.view_account().await?.balance;

    // alice requests near -> success
    let res_alice_one = alice
        .call(&contract.id(), "request_near")
        .args_json((alice.id(), U128::from(parse_near!("20 N"))))
        .transact()
        .await?;

    assert!(res_alice_one.is_success());
    assert!(contract.view_account().await?.balance < initial_faucet_balance);
    assert!(alice.view_account().await?.balance > initial_alice_balance);

    // alice tries to request again without waiting 1hour -> fails
    let res_alice_two = alice
        .call(&contract.id(), "request_near")
        .args_json((alice.id(), U128::from(parse_near!("20 N"))))
        .transact()
        .await?;

    assert!(res_alice_two.is_failure());

    // ############################
    // Omitting this asserts as it takes forever to run

    // alice requests near again after > 1hour -> success
    // we advance to more blocks than the REQUEST_GAP_LIMITER set in the settings.rs
    // worker.fast_forward(3700000).await?;

    // let res_alice_three = alice
    //     .call(&contract.id(), "request_near")
    //     .args_json((alice.id(), U128::from(parse_near!("20 N"))))
    //     .transact()
    //     .await?;

    // assert!(res_alice_three.is_success());
    // ############################

    // bob tries to request too much near -> fails
    let res_bob_one = alice
        .call(&contract.id(), "request_near")
        .args_json((bob.id(), U128::from(parse_near!("30 N"))))
        .transact()
        .await?;

    assert!(res_bob_one.is_failure());

    Ok(())
}

#[tokio::test]
async fn test_request_additional_liquidity() -> anyhow::Result<()> {
    let (faucet_contract, alice, worker) = init().await?;
    let va

    Ok(())
}
