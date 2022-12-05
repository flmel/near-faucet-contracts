use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen, require,
    store::{LookupSet, UnorderedMap, Vector},
    AccountId, Balance, BorshStorageKey, Promise,
};

mod external;
mod fungible_tokens;
mod settings;

use crate::fungible_tokens::*;
use crate::settings::*;
use external::vault_contract;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    recent_contributions: Vector<(AccountId, Balance)>,
    recent_receivers: UnorderedMap<AccountId, u64>,
    ft_faucet: UnorderedMap<AccountId, FTconfig>,
    blocklist: LookupSet<AccountId>,
    factory_list: LookupSet<AccountId>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    RecentContributions,
    RecentReceivers,
    FTFaucet,
    Blocklist,
    FactoryList,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            recent_contributions: Vector::new(StorageKey::RecentContributions),
            recent_receivers: UnorderedMap::new(StorageKey::RecentReceivers),
            ft_faucet: UnorderedMap::new(StorageKey::FTFaucet),
            blocklist: LookupSet::new(StorageKey::Blocklist),
            factory_list: LookupSet::new(StorageKey::FactoryList),
        }
    }
}

#[near_bindgen]
impl Contract {
    // Request NEAR from the faucet
    pub fn request_near(&mut self, receiver_id: AccountId, amount: U128) {
        // Check if the receiver is in the blocklist
        require!(
            self.blocklist.contains(&env::predecessor_account_id()) == false,
            "Account has been blocklisted!"
        );
        require!(
            amount.0 <= MAX_WITHDRAW_AMOUNT,
            "Withdraw request too large!"
        );

        // Remove expired restrictions
        self.remove_expired_restrictions();

        let current_timestamp_ms: u64 = env::block_timestamp_ms();
        // did the receiver get money recently? if not insert them in the the map
        match self.recent_receivers.get(&receiver_id) {
            Some(previous_timestamp_ms) => {
                // if they did receive within the last ~30 min block them
                if &current_timestamp_ms - previous_timestamp_ms < REQUEST_GAP_LIMITER {
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
        // make the transfer
        Promise::new(receiver_id.clone()).transfer(amount.0);
        // check if additional liquidity is needed
        if env::account_balance() < MIN_BALANCE_THRESHOLD {
            self.request_additional_liquidity();
        }
    }

    fn remove_expired_restrictions(&mut self) {
        let mut to_del: Vec<AccountId> = vec![];

        for (receiver_id, timestamp) in self.recent_receivers.iter() {
            if env::block_timestamp_ms() - timestamp > REQUEST_GAP_LIMITER {
                to_del.push(receiver_id.clone());
            }
        }

        for receiver_id in to_del {
            self.recent_receivers.remove(&receiver_id);
        }
    }

    #[private]
    pub fn add_to_blocklist(&mut self, account_id: AccountId) {
        self.blocklist.insert(account_id);
    }

    #[private]
    pub fn batch_add_to_blocklist(&mut self, accounts: Vec<AccountId>) {
        self.blocklist.extend(accounts);
    }

    #[private]
    pub fn remove_from_blocklist(&mut self, account_id: AccountId) {
        self.blocklist.remove(&account_id);
    }

    #[private]
    pub fn clear_recent_receivers(&mut self) {
        self.recent_receivers.clear();
    }

    // // contribute to the faucet contract to get in the list of fame
    // #[payable]
    // pub fn contribute(&mut self) {
    //     let contributor: AccountId = env::predecessor_account_id();
    //     let amount: Balance = env::attached_deposit();

    //     self.recent_contributions.set(0, (contributor, amount));
    //     self.recent_contributions.iter().take(10);
    // }

    // // get top contributors
    // pub fn get_recent_contributions(&self) -> Vec<(AccountId, String)> {
    //     self.recent_contributions
    //         .iter()
    //         .map(|(account_id, amount)| (account_id.clone(), amount.to_string()))
    //         .collect()
    // }

    // request_additional_liquidity
    fn request_additional_liquidity(&self) {
        vault_contract::ext(VAULT_ID.parse().unwrap()).request_funds();
    }
}
