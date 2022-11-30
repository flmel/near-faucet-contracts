use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupSet,
    env,
    json_types::U128,
    near_bindgen, require, AccountId, Balance, Promise,
};

mod external;
mod fungible_tokens;
mod settings;

use crate::fungible_tokens::*;
use crate::settings::*;
use external::vault_contract;
use std::collections::HashMap;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    recent_contributions: Vec<(AccountId, Balance)>,
    recent_receivers: HashMap<AccountId, u64>,
    ft_faucet: HashMap<AccountId, FTconfig>,
    blacklist: LookupSet<AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            recent_contributions: Vec::new(),
            recent_receivers: HashMap::new(),
            ft_faucet: HashMap::new(),
            blacklist: LookupSet::new(b"s"),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn request_funds(&mut self, receiver_id: AccountId, amount: U128) {
        // check if predecessor is in the blacklist
        require!(
            self.blacklist.contains(&env::predecessor_account_id()) == false,
            "Account has been blacklisted!"
        );
        require!(
            amount.0 <= MAX_WITHDRAW_AMOUNT,
            "Withdraw request too large!"
        );

        let current_timestamp_ms: u64 = env::block_timestamp_ms();

        // purge expired restrictions
        self.recent_receivers
            .retain(|_, v: &mut u64| *v + REQUEST_GAP_LIMITER > current_timestamp_ms);

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

    // #[private] this macro does not expand for unit testing therefore I'm ignoring it for the time being
    pub fn add_to_blacklist(&mut self, account_id: AccountId) {
        assert_self();
        self.blacklist.insert(&account_id);
    }

    pub fn batch_add_to_blacklist(&mut self, accounts: Vec<AccountId>) {
        assert_self();
        // sadly no append TODO: Optimise
        for account in accounts {
            self.blacklist.insert(&account);
        }
    }

    // #[private] this macro does not expand for unit testing therefore I'm ignoring it for the time being
    pub fn remove_from_blacklist(&mut self, account_id: AccountId) {
        assert_self();
        self.blacklist.remove(&account_id);
    }

    // #[private] this macro does not expand for unit testing therefore I'm ignoring it for the time being
    pub fn clear_recent_receivers(&mut self) {
        assert_self();
        self.recent_receivers.clear();
    }

    // contribute to the faucet contract to get in the list of fame
    #[payable]
    pub fn contribute(&mut self) {
        let contributor: AccountId = env::predecessor_account_id();
        let amount: Balance = env::attached_deposit();

        self.recent_contributions.insert(0, (contributor, amount));
        self.recent_contributions.truncate(10);
    }

    // get top contributors
    pub fn get_recent_contributions(&self) -> Vec<(AccountId, String)> {
        self.recent_contributions
            .iter()
            .map(|(account_id, amount)| (account_id.clone(), amount.to_string()))
            .collect()
    }

    // request_additional_liquidity
    fn request_additional_liquidity(&self) {
        vault_contract::ext(VAULT_ID.parse().unwrap()).request_funds();
    }
}
