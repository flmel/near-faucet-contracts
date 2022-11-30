# Near Testnet Faucet

Near Testnet Faucet consists of two Smart Contracts witten in Rust and a [TailwindCSS](https://tailwindcss.com/) and [AlpineJs](https://alpinejs.dev/) frontend, currently deployed at https://near-faucet.io. It aims to help developers coming from other blockchains who are used to the concept of *Faucets* and people who for some reason are in need of _Testnet_ Near.

### Prerequisites

In order to compile and run everything you will need:

* Node and [near-cli](https://github.com/near/near-cli) installed
* Rust and WASM toolchain [detailed steps here](https://www.near-sdk.io/)


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
79 // if env::account_balance() <MIN_BALANCE_THRESHOLD {
80 //   self.request_additional_liquidity();
81 //}
```

#### build:  
`RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release`

#### deploy:  
`near deploy --wasmFile PATH_TO.wasm --accountId ACCOUNT_YOU_HAVE_KEYS_FOR`

Alternatively, you can make use of `near dev deploy`


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

## Testing
Currently the project makes use of Rusts Unit testing (ish), Integration tests are a bit hard since the tooling is under restructuring/refactoring at the moment.    
test:  
`cargo test `

## Frontend
Frontend consists of a static web app built with [TailwindCSS](https://tailwindcss.com/), [AlpineJs](https://alpinejs.dev/) and [near-api-js](https://github.com/near/near-api-js) which can be found on the [frontend](https://github.com/flmel/near-testnet-faucet/tree/frontend) branch.

### Further development and research/exploration

- [ ] Add [workspaces-rs](https://github.com/near/workspaces-rs/) integration tests. (currently blocked by: [#110](https://github.com/near/workspaces-rs/issues/110))
- [ ] Make the contract emit custom [events](https://nomicon.io/Standards/EventsFormat)
- [ ] Move the frontend to [Yew](https://yew.rs/)
- [ ] Improve defensive mechanics
- [ ] Stake percentage of the vault/account balance with a testnet validator  
- [ ] Add ability to request USN (either trough XCC to usdn.testnet or via contract ballance support)
- [ ] Explore the idea to support other FTs
    - maybe airdrop mechanics(via collaborative effort) to some kind of Dev FT/NFT holders


### Useful Links

* [Near University](https://near.university)
* [Near University Discord](https://discord.gg/k4pxafjMWA)
* [Near Docs](https://docs.near.org)
* [Near SDK-RS Docs](https://near-sdk.io)
* [Testnet Blockchain Explorer](https://explorer.testnet.near.org/)
