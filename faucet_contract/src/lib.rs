// Find all our documentation at https://docs.near.org
use near_sdk::{
    env, near, require,
    store::{IterableMap, LookupSet, Vector},
    AccountId, BorshStorageKey, Gas, NearToken, Promise,
};

use regex::Regex;

const NO_DEPOSIT: NearToken = NearToken::from_near(0);
const NO_ARGS: Vec<u8> = vec![];

// Define the contract structure
#[near(contract_state)]
pub struct Contract {
    recent_contributions: Vector<(AccountId, NearToken)>,
    recent_receivers: IterableMap<AccountId, u64>,
    successful_requests: u64,
    // ft_faucet: IterableMap<AccountId, FTconfig>,
    blacklist: LookupSet<AccountId>,
    // factory_list: LookupSet<AccountId>,
    mod_list: LookupSet<AccountId>,
    vault_contract_id: AccountId,
    min_balance_threshold: NearToken,
    request_allowance: NearToken,
    request_gap_required: u64,
}

#[near(serializers = [json])]
pub struct Stats {
    successful_requests: u64,
    ft_contracts_listed: u64,
    recent_contributions: Vec<(AccountId, NearToken)>,
}

#[near(serializers=[borsh])]
#[derive(BorshStorageKey)]
enum StorageKey {
    RecentContributions,
    RecentReceivers,
    // FTFaucet,
    Blacklist,
    ModList,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            recent_contributions: Vector::new(StorageKey::RecentContributions),
            recent_receivers: IterableMap::new(StorageKey::RecentReceivers),
            // ft_faucet: IterableMap::new(StorageKey::FTFaucet),
            successful_requests: 0,
            blacklist: LookupSet::new(StorageKey::Blacklist),
            mod_list: LookupSet::new(StorageKey::ModList),
            vault_contract_id: "vault.nonofficial.testnet".parse().unwrap(),
            min_balance_threshold: NearToken::from_near(10_000),
            request_allowance: NearToken::from_near(10),
            request_gap_required: 3_600_000,
        }
    }
}

// Implement the contract structure
#[near]
impl Contract {
    // Public method - returns the greeting saved, defaulting to DEFAULT_GREETING
    pub fn request_near(&mut self, receiver_id: AccountId, request_amount: NearToken) {
        // check if the receiver is in the blacklist
        require!(
            self.blacklist.contains(&receiver_id) == false,
            "Account has been blacklisted!"
        );
        require!(
            request_amount <= self.request_allowance,
            "Withdraw request too large!"
        );
        let pattern = Regex::new(r"^([A-Za-z\d]+[\-_])*[A-Za-z\d]+\.testnet$").unwrap();
        require!(
            pattern.is_match(&receiver_id.to_string() as &str),
            "Invalid receiver account id!"
        );

        // remove expired restrictions
        self.remove_expired_restrictions();
        // check if the receiver has requested recently
        self.check_recent_receivers(&receiver_id);

        // make the transfer
        Promise::new(receiver_id.clone()).transfer(request_amount);
        // increment the successful requests
        self.successful_requests += 1;
        // check if additional liquidity is needed
        if env::account_balance() < self.min_balance_threshold {
            self.request_additional_liquidity();
        }
    }

    #[private]
    pub fn clear_recent_receivers(&mut self) {
        self.recent_receivers.clear();
    }

    fn remove_expired_restrictions(&mut self) {
        let mut to_del: Vec<AccountId> = vec![];

        for (receiver_id, timestamp) in self.recent_receivers.iter() {
            if env::block_timestamp_ms() - timestamp > self.request_gap_required {
                to_del.push(receiver_id.clone());
            }
        }

        for receiver_id in to_del {
            self.recent_receivers.remove(&receiver_id);
        }
    }

    fn request_additional_liquidity(&self) {
        Promise::new(self.vault_contract_id.clone()).function_call(
            "request_funds".to_string(),
            NO_ARGS,
            NO_DEPOSIT,
            Gas::from_tgas(5),
        );
    }

    fn check_recent_receivers(&mut self, receiver_id: &AccountId) {
        let current_timestamp_ms: u64 = env::block_timestamp_ms();
        // did the receiver get money recently? if not insert them in the the map
        match self.recent_receivers.get(receiver_id) {
            Some(previous_timestamp_ms) => {
                // if they did receive within the last ~30 min block them
                if &current_timestamp_ms - previous_timestamp_ms < self.request_gap_required {
                    env::panic_str(
                        "You have to wait for a little longer before requesting to this account!",
                    )
                }
            }
            None => {
                self.recent_receivers
                    .insert(receiver_id.clone(), current_timestamp_ms);
            }
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn get_default_greeting() {
    //     let contract = Contract::default();
    //     // this test did not call set_greeting so should return the default "Hello" greeting
    //     assert_eq!(contract.get_greeting(), "Hello");
    // }

    // #[test]
    // fn set_then_get_greeting() {
    //     let mut contract = Contract::default();
    //     contract.set_greeting("howdy".to_string());
    //     assert_eq!(contract.get_greeting(), "howdy");
    // }
}
