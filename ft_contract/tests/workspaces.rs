use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::json_types::U128;
use near_sdk::ONE_YOCTO;
use near_units::parse_near;
use workspaces::operations::Function;
use workspaces::result::ValueOrReceiptId;
use workspaces::{Account, AccountId, Contract, DevNetwork, Worker};

async fn register_user(contract: &Contract, account_id: &AccountId) -> anyhow::Result<()> {
    let res = contract
        .call("storage_deposit")
        .args_json((account_id, Option::<bool>::None))
        .max_gas()
        .deposit(near_sdk::env::storage_byte_cost() * 125)
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}

async fn init(
    worker: &Worker<impl DevNetwork>,
    initial_balance: U128,
) -> anyhow::Result<(Contract, Account, Contract)> {
    let ft_contract = worker
        .dev_deploy(include_bytes!("../res/ft_contract.wasm"))
        .await?;

    let metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "Test Token".to_string(),
        symbol: "TEST".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 24,
    };

    let res = ft_contract
        .call("new")
        .args_json((ft_contract.id(), initial_balance, metadata))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let faucet_contract = worker
        .dev_deploy(include_bytes!(
            "../../faucet_contract/res/faucet_contract.wasm"
        ))
        .await?;

    let alice = ft_contract
        .as_account()
        .create_subaccount("alice")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;
    register_user(&ft_contract, alice.id()).await?;

    let res = ft_contract
        .call("storage_deposit")
        .args_json((alice.id(), Option::<bool>::None))
        .deposit(near_sdk::env::storage_byte_cost() * 125)
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    return Ok((ft_contract, alice, faucet_contract));
}

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("10000 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, _, _) = init(&worker, initial_balance).await?;

    let res = contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?, initial_balance);

    Ok(())
}

#[tokio::test]
async fn simple_transfer() -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("10000 N"));
    let transfer_amount = U128::from(parse_near!("100 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, alice, _) = init(&worker, initial_balance).await?;

    let res = contract
        .call("ft_transfer")
        .args_json((alice.id(), transfer_amount, Option::<bool>::None))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let root_balance = contract
        .call("ft_balance_of")
        .args_json((contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let alice_balance = contract
        .call("ft_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0 - transfer_amount.0, root_balance.0);
    assert_eq!(transfer_amount.0, alice_balance.0);

    Ok(())
}

#[tokio::test]
async fn simulate_transfer_call_with_burned_amount() -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("10000 N"));
    let transfer_amount = U128::from(parse_near!("100 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, _, faucet_contract) = init(&worker, initial_balance).await?;

    // faucet contract must be registered as a FT account
    register_user(&contract, faucet_contract.id()).await?;

    // ft_contract invests in faucet_contract by calling `ft_transfer_call`
    let res = contract
        .batch()
        .call(
            Function::new("ft_transfer_call")
                .args_json((
                    faucet_contract.id(),
                    transfer_amount,
                    Option::<String>::None,
                    "10",
                ))
                .deposit(ONE_YOCTO)
                .gas(300_000_000_000_000 / 2),
        )
        .call(
            Function::new("storage_unregister")
                .args_json((Some(true),))
                .deposit(ONE_YOCTO)
                .gas(300_000_000_000_000 / 2),
        )
        .transact()
        .await?;
    println!("res: {:#?}", res);
    // assert!(res.is_success());

    // let logs = res.logs();
    // let expected = format!("Account @{} burned {}", contract.id(), 10);
    // assert!(logs.len() >= 2);
    // assert!(logs.contains(&"The account of the sender was deleted"));
    // assert!(logs.contains(&(expected.as_str())));

    // let res = contract.call("ft_total_supply").view().await?;
    // assert_eq!(res.json::<U128>()?.0, transfer_amount.0 - 10);
    // let faucet_balance = contract
    //     .call("ft_balance_of")
    //     .args_json((faucet_contract.id(),))
    //     .view()
    //     .await?
    //     .json::<U128>()?;
    // assert_eq!(faucet_balance.0, transfer_amount.0 - 10);

    Ok(())
}

#[tokio::test]
async fn simulate_transfer_call_with_immediate_return_and_no_refund() -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("10000 N"));
    let transfer_amount = U128::from(parse_near!("100 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, _, defi_contract) = init(&worker, initial_balance).await?;

    // defi contract must be registered as a FT account
    register_user(&contract, defi_contract.id()).await?;

    // root invests in defi by calling `ft_transfer_call`
    let res = contract
        .call("ft_transfer_call")
        .args_json((
            defi_contract.id(),
            transfer_amount,
            Option::<String>::None,
            "take-my-money",
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let root_balance = contract
        .call("ft_balance_of")
        .args_json((contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let defi_balance = contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0 - transfer_amount.0, root_balance.0);
    assert_eq!(transfer_amount.0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn simulate_transfer_call_when_called_contract_not_registered_with_ft() -> anyhow::Result<()>
{
    let initial_balance = U128::from(parse_near!("10000 N"));
    let transfer_amount = U128::from(parse_near!("100 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, _, faucet_contract) = init(&worker, initial_balance).await?;

    // call fails because faucet contract is not registered as FT user
    let res = contract
        .call("ft_transfer_call")
        .args_json((
            faucet_contract.id(),
            transfer_amount,
            Option::<String>::None,
            "take-my-money",
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_failure());

    // balances remain unchanged
    let root_balance = contract
        .call("ft_balance_of")
        .args_json((contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let faucet_balance = contract
        .call("ft_balance_of")
        .args_json((faucet_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0, root_balance.0);
    assert_eq!(0, faucet_balance.0);

    Ok(())
}

#[tokio::test]
async fn simulate_transfer_call_with_promise_and_refund() -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("10000 N"));
    let refund_amount = U128::from(parse_near!("50 N"));
    let transfer_amount = U128::from(parse_near!("100 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, _, faucet_contract) = init(&worker, initial_balance).await?;

    // faucet contract must be registered as a FT account
    register_user(&contract, faucet_contract.id()).await?;

    let res = contract
        .call("ft_transfer_call")
        .args_json((
            faucet_contract.id(),
            transfer_amount,
            Option::<String>::None,
            refund_amount.0.to_string(),
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let root_balance = contract
        .call("ft_balance_of")
        .args_json((contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let faucet_balance = contract
        .call("ft_balance_of")
        .args_json((faucet_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(
        initial_balance.0 - transfer_amount.0 + refund_amount.0,
        root_balance.0
    );
    assert_eq!(transfer_amount.0 - refund_amount.0, faucet_balance.0);

    Ok(())
}

#[tokio::test]
async fn simulate_transfer_call_promise_panics_for_a_full_refund() -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("10000 N"));
    let transfer_amount = U128::from(parse_near!("100 N"));
    let worker = workspaces::sandbox().await?;
    let (contract, _, defi_contract) = init(&worker, initial_balance).await?;

    // defi contract must be registered as a FT account
    register_user(&contract, defi_contract.id()).await?;

    // root invests in defi by calling `ft_transfer_call`
    let res = contract
        .call("ft_transfer_call")
        .args_json((
            defi_contract.id(),
            transfer_amount,
            Option::<String>::None,
            "no parsey as integer big panic oh no".to_string(),
        ))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let promise_failures = res.receipt_failures();
    assert_eq!(promise_failures.len(), 1);
    let failure = promise_failures[0].clone().into_result();
    if let Err(err) = failure {
        assert!(err.to_string().contains("ParseIntError"));
    } else {
        unreachable!();
    }

    // balances remain unchanged
    let root_balance = contract
        .call("ft_balance_of")
        .args_json((contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let defi_balance = contract
        .call("ft_balance_of")
        .args_json((defi_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance, root_balance);
    assert_eq!(0, defi_balance.0);

    Ok(())
}
