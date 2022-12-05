Faucet Contract
===================
This is the faucet contract currently deployed on https://near-faucet.io it [factory](../factory_contract/). 

*Be advised that this is non audited contract for educational purposes only*


## Deployment and Usage
Before building and deploying you should change to these contract settings (found in lib.rs) with the following snippet
```rust
// sets the maximum amount an user can request to 10 near
const MAX_WITHDRAW_AMOUNT: Balance = 10 * ONE_NEAR;
// sets the the time (in ms) that user shall wait before subsequent request to 1 min
const REQUEST_GAP_LIMITER: u64 = 60000;
// sets the vault contract (you cna leave that if you don't plan to deploy the one found int the vault branch)
const VAULT_ID: &str = "vault.nonofficial.testnet";
// sets the balance threshold required to make a call to the vault contract for additional liquidity
const MIN_BALANCE_THRESHOLD: Balance = 10 * ONE_NEAR;
```

If you want to test/experiment without using the vault contract you should omit the `env::account_balance()` check at the end of the `request_funds` fn

```rust
78 // check if additional liquidity is needed
79 // if env::account_balance() < MIN_BALANCE_THRESHOLD {
80 //   self.request_additional_liquidity();
81 // }
```

#### Brief overview of the contracts functions

```rust 
pub fn request_funds(...) {
// requests funds to be sent to certain receiver_id
}
pub fn contribute(...) {
// records the contributor to the contributors (sorts the Vec before inserting)... 
}
pub fn get_top_contributors(...) {
// retrieves the top ten contributors
}
pub fn add_to_blacklist(...) {
// adds an AccountId to the blacklist
}
pub fn remove_from_blacklist(...) {
// removes an AccountId from the blacklist
} 
pub fn clear_recent_receivers(...) {
// clears the recent_receivers map, thus removing all current time constrains 
}
fn request_additional_liquidity(...) {
// this makes XCC to an vault contract (can be found in vault branch) if the faucets account balance goes bellow certain threshold 
}
```

___
[NEAR](https://near.org) - [NEAR Docs](https://near.org) - [Nomicon](https://nomicon.io) - [Discord](https://near.chat) - [AwesomeNear](https://awesomenear.com)
