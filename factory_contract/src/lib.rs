use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::Vector,
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, Gas, Promise, PromiseError, ONE_NEAR,
};

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
    pub fn create_contract(
        &mut self,
        desired_prefix: String,
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Promise {
        assert_self();

        let ft_contract_id: AccountId = format!("{}.{}", desired_prefix, env::current_account_id())
            .parse()
            .unwrap();

        Promise::new(ft_contract_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(3 * ONE_NEAR)
            .deploy_contract(CODE.to_vec())
            .then(ft_contract::ext(ft_contract_id.clone()).new(owner_id, total_supply, metadata))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * 10u64.pow(12))) // 5*10e12, 5Tgas
                    .save_contract_callback(ft_contract_id),
            )
    }

    // Add contract to the list of deployed contracts
    pub fn add_contract(&mut self, ft_contract_id: AccountId) {
        assert_self();
        self.deployed_contracts.push(&ft_contract_id);
    }

    // Remove the contract from the list of deployed contracts
    pub fn remove_contract(&mut self, ft_contract_id: AccountId) {
        assert_self();

        let index = self
            .deployed_contracts
            .iter()
            .position(|id| id == ft_contract_id);

        if let Some(index) = index {
            self.deployed_contracts.swap_remove(index as u64);
        }
    }

    pub fn delete_contract_account(&mut self, ft_contract_id: AccountId) {
        assert_self();

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
    pub fn save_contract_callback(
        &mut self,
        ft_contract_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        assert_self();

        // check if the promise failed
        if call_result.is_err() {
            log!("Create contract failed!");
            return;
        }

        // add the contract to the list of deployed contracts
        self.add_contract(ft_contract_id);
    }

    // Remove contract callback
    pub fn remove_contract_callback(
        &mut self,
        ft_contract_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        assert_self();

        // check if the promise failed
        if call_result.is_err() {
            log!("Delete contract failed!");
            return;
        }

        // remove the contract from the list of deployed contracts
        self.remove_contract(ft_contract_id);
    }
}

// UNIT TESTS
// Note: #[private] macro doesn't expand in unit tests
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .is_view(is_view)
            .current_account_id("contract.testnet".parse().unwrap());
        builder
    }

    #[test]
    #[should_panic]
    fn panics_test_add_contract() {
        let mut contract = Contract::default();
        contract.add_contract(accounts(0));
    }

    #[test]
    fn test_add_contract() {
        let mut context = get_context(false);

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        let mut contract = Contract::default();
        contract.add_contract(accounts(0));

        assert_eq!(contract.num_contracts(), 1);
    }

    #[test]
    #[should_panic]
    fn panics_test_remove_contract() {
        let mut contract = Contract::default();
        contract.remove_contract(accounts(0));
    }

    #[test]
    fn test_remove_contract() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        contract.add_contract(accounts(0));

        contract.remove_contract(accounts(0));

        assert_eq!(contract.num_contracts(), 0);
    }
}
