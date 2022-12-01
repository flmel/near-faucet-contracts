use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::Vector,
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, Gas, Promise, PromiseError,
};

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
// Compiled ft_contract from https://github.com/flmel/near-faucet-contracts
const CODE: &[u8] = include_bytes!("./ft.wasm");

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    deployed_contracts: Vector<AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            deployed_contracts: Vector::new(b'd'),
        }
    }
}

// Interface for cross-contract calls to the ft_contract
#[ext_contract(ft_contract)]
trait FtContract {
    fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata);
    fn delete_contract_account();
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn create_contract(
        &mut self,
        desired_prefix: String,
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Promise {
        let ft_contract_id: AccountId = format!("{}.{}", desired_prefix, env::current_account_id())
            .parse()
            .unwrap();

        Promise::new(ft_contract_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(3_000_000_000_000_000_000_000_000) // 3e24yN, 3N
            .deploy_contract(CODE.to_vec())
            .then(ft_contract::ext(ft_contract_id.clone()).new(owner_id, total_supply, metadata))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * 10u64.pow(12))) // 5*10e12, 5Tgas
                    .save_contract_callback(ft_contract_id),
            )
    }

    // Private
    // Add contract to the list of deployed contracts
    #[private]
    pub fn add_contract(&mut self, ft_contract_id: AccountId) {
        self.deployed_contracts.push(&ft_contract_id);
    }
    // Remove the contract from the list of deployed contracts
    #[private]
    pub fn remove_contract(&mut self, ft_contract_id: AccountId) {
        let index = self
            .deployed_contracts
            .iter()
            .position(|id| id == ft_contract_id);
        if let Some(index) = index {
            self.deployed_contracts.swap_remove(index as u64);
        }
    }
    #[private]
    pub fn delete_contract_account(&mut self, ft_contract_id: AccountId) {
        ft_contract::ext(ft_contract_id.clone())
            .with_static_gas(Gas(5 * 10u64.pow(12)))
            .delete_contract_account()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * 10u64.pow(12))) // 5*10e12, 5Tgas
                    .remove_contract_callback(ft_contract_id),
            );
    }

    // Return number of contracts deployed
    pub fn num_contracts(&self) -> u64 {
        self.deployed_contracts.len() as u64
    }

    // Callbacks
    // Save contract callback
    #[private]
    pub fn save_contract_callback(
        &mut self,
        ft_contract_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        // Check if the promise failed
        if call_result.is_err() {
            log!("Create contract failed!");
            return;
        }

        // Add the contract to the list of deployed contracts
        self.add_contract(ft_contract_id);
    }
    // Remove contract callback
    #[private]
    pub fn remove_contract_callback(
        &mut self,
        ft_contract_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        // Check if the promise failed
        if call_result.is_err() {
            log!("Delete contract failed!");
            return;
        }

        // Remove the contract from the list of deployed contracts
        self.remove_contract(ft_contract_id);
    }
}
