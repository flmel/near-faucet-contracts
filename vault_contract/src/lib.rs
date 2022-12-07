use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen, require,
    store::LookupSet,
    AccountId, BorshStorageKey, Promise, ONE_NEAR,
};

// 24h in ms
const REQUEST_COOLDOWN_MS: u64 = 86_400_000;
// 10_000 NEAR
const AMOUNT_TO_BE_SENT: u128 = 10_000 * ONE_NEAR;

// Contract struct
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    last_successful_call: u64,
    whitelist: LookupSet<AccountId>,
}

// Storage keys
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Whitelist,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_successful_call: 0,
            whitelist: LookupSet::new(StorageKeys::Whitelist),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn request_funds(&mut self) {
        // Require the predecessor to be in the allowlist
        require!(
            self.whitelist.contains(&env::predecessor_account_id()),
            "Sorry, you are not allowed to request funds!"
        );
        // Require that the REQUEST_COOLDOWN_MS has passed
        require!(
            env::block_timestamp_ms() - self.last_successful_call > REQUEST_COOLDOWN_MS,
            "Cooldown haven't passed"
        );

        // Make the transfer
        Promise::new(env::predecessor_account_id()).transfer(AMOUNT_TO_BE_SENT);
        // Update last_call
        self.last_successful_call = env::block_timestamp_ms();
    }

    // Add an account to the whitelist
    #[private]
    pub fn add_to_whitelist(&mut self, account_id: AccountId) {
        self.whitelist.insert(account_id);
    }

    // Remove an account from the whitelist
    #[private]
    pub fn remove_from_whitelist(&mut self, account_id: AccountId) {
        self.whitelist.remove(&account_id);
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
    fn add_to_whitelist() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        contract.add_to_whitelist(accounts(0));

        // Alice shall be in the whitelist
        assert!(contract.whitelist.contains(&accounts(0)));
    }

    #[test]
    fn remove_from_whitelist() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());
        // Add Alice to the whitelist
        contract.add_to_whitelist(accounts(0));
        // Remove Alice from the whitelist
        contract.remove_from_whitelist(accounts(0));

        // Alice shall be removed from the whitelist
        assert!(contract.whitelist.contains(&accounts(0)) == false);
    }
}
