FT Factory Contract
===================
This contract acts as a factory. It creates desired subaccount and deploys precompiled FT contracts with user provided metadata


Usage Example
---

Create mytoken.factory.testnet and deploy FT contract with the provided metadata

    near call $CONTRACT create_contract '{"desired_prefix": "mytoken", "owner_id": "myaccount.testnet", "total_supply": "100000000000000", "metadata": { "spec": "ft-1.0.0", "name": "myAwesomeToken", "symbol": "MAT", "decimals": 24 }}' --account_id $CONTRACT --gas 300000000000000

Get the amount of ft contracts currently deployed by the factory
    
    near view $CONTRACT get_num_contracts