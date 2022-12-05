near-faucet-contracts
====
Includes the contracts currently in use by [near-faucet.io](https://near-faucet.io) representing faucet, vault, FT contract, and a FT contract factory.

*Be advised that those are non audited contracts for educational purposes only*

 * **[factory_contract](factory_contract)** - factory that allows users to create sub-accounts and deploy precompiled FT contract with provided metadata 
 * **[faucet_contract](faucet_contract)** - token faucet (FT and Native Near)
 * **[ft_contract](ft_contract)** - an FT contract that's being deployed by the factory contract 
 * **[vault_contract](vault_contract)** - vault to store the surplus of native tokens in the faucet


In order to compile and run everything you will need:

* Node and [near-cli](https://github.com/near/near-cli) installed
* Rust and WASM toolchain [steps here](https://docs.near.org/sdk/rust/introduction)

 
___
[NEAR](https://near.org) - [NEAR Docs](https://near.org) - [Nomicon](https://nomicon.io) - [Discord](https://near.chat) - [AwesomeNear](https://awesomenear.com)
