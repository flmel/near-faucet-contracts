use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen, require,
    store::LookupSet,
    AccountId, BorshStorageKey, Promise, ONE_NEAR,
};

// 24h in ms
const REQUEST_COOLDOWN_MS: u64 = 86_400_000;
// because why always round numbers
const AMOUNT_TO_BE_SENT: u128 = 7349 * ONE_NEAR;
// Contract struct
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    last_successful_call: u64,
    allow_list: LookupSet<AccountId>,
}
// Storage keys
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    AllowList,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_successful_call: 0,
            allow_list: LookupSet::new(StorageKeys::AllowList),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn request_funds(&mut self) {
        // require the predecessor to be in the allowlist
        require!(
            self.allow_list.contains(&env::predecessor_account_id()),
            "Sorry, you are not allowed to request funds!"
        );
        // require that the REQUEST_COOLDOWN_MS has passed
        require!(
            env::block_timestamp_ms() - self.last_successful_call > REQUEST_COOLDOWN_MS,
            "Cooldown haven't passed"
        );
        // make the transfer
        Promise::new(env::predecessor_account_id()).transfer(AMOUNT_TO_BE_SENT);
        // update last_call
        self.last_successful_call = env::block_timestamp_ms();
    }
    #[private]
    pub fn add_to_allow_list(&mut self, account_id: AccountId) {
        self.allow_list.insert(account_id);
    }
    #[private]
    pub fn remove_from_allowlist(&mut self, account_id: AccountId) {
        self.allow_list.remove(&account_id);
    }
}
