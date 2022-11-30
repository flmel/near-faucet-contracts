use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupSet,
    env, near_bindgen, require, AccountId, Promise, ONE_NEAR,
};

// 24h in ms
const REQUEST_COOLDOWN_MS: u64 = 86400000;
// because why always round numbers
const AMOUNT_TO_BE_SENT: u128 = 7349 * ONE_NEAR;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    last_call: u64,
    whitelist: LookupSet<AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            last_call: 0,
            whitelist: LookupSet::new(b"s"),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn request_funds(&mut self) {
        // require the predecessor to be whitelisted
        require!(
            self.whitelist.contains(&env::predecessor_account_id()),
            "Sorry, you are not allowed to request funds!"
        );
        // require that the REQUEST_COOLDOWN_MS has passed
        require!(
            env::block_timestamp_ms() - self.last_call > REQUEST_COOLDOWN_MS,
            "Cooldown haven't passed"
        );
        // make the transfer
        Promise::new(env::predecessor_account_id()).transfer(AMOUNT_TO_BE_SENT);
        // update last_call
        self.last_call = env::block_timestamp_ms();
    }

    pub fn add_to_whitelist(&mut self, account_id: AccountId) {
        assert_self();
        self.whitelist.insert(&account_id);
    }

    pub fn remove_from_whitelist(&mut self, account_id: AccountId) {
        assert_self();
        self.whitelist.remove(&account_id);
    }
}
