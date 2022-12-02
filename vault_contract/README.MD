Vault Contract
===================
This is an simple vault contract that's being called by the [faucet](../faucet_contract/) when the faucet liquidity drops bellow certain threshold.
After a successful liquidity request, the contract will deny any consecutive requests for 24 hours. This increases the time for the faucet contract  to be drained. 

*Be advised that this is non audited contract for educational purposes only*


#### Functions overview

```rust
pub fn request_funds() {
// Requests additional liquidity 
}
pub fn add_to_allow_list() {
// Adds an AccountId to the list of contracts that's able to require predefined liquidity 
}
pub fn remove_from_allowlist() {
// Removes an AccountId from the allow list
}
```

___
[NEAR](https://near.org) - [NEAR Docs](https://near.org) - [Nomicon](https://nomicon.io) - [Discord](https://near.chat) - [AwesomeNear](https://awesomenear.com)
