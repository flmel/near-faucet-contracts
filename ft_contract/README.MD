FT Contract
===================
This is an FT contract that's being deployed by the [factory](../factory_contract/). 

*Be advised that this is non audited contract for educational purposes only*


#### Functions overview

Apart of FT Core - [NEP-141](https://github.com/near/NEPs/blob/master/neps/nep-0141.md) and FT Metadata - [NEP-148](https://github.com/near/NEPs/blob/master/neps/nep-0148.md). This contract implements the following functions

```rust
pub fn add_mod(account_id) {
// Add account to the mod list
}
pub fn get_registered_accounts() {
// Get registered accounts count
}
pub fn list_on_faucet(faucet_account_id, ft_request_allowance, ft_initial_balance) {
// Add the token to the faucet contract
}
pub fn remove_from_faucet(faucet_account_id, confirm) {
// Remove the token from the faucet contract
}
pub fn delete_contract_account() {
// Delete the contract account
// Self destruct !!!
}
```

___
[NEAR](https://near.org) - [NEAR Docs](https://near.org) - [Nomicon](https://nomicon.io) - [Discord](https://near.chat) - [AwesomeNear](https://awesomenear.com)
