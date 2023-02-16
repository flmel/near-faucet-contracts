use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen, require,
    serde::{Deserialize, Serialize},
    store::{LookupSet, UnorderedMap},
    AccountId, Balance, BorshStorageKey, Promise,
};

mod external;
mod fungible_tokens;

use crate::fungible_tokens::*;
use external::vault_contract;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    recent_contributions: Vec<(AccountId, Balance)>,
    recent_receivers: UnorderedMap<AccountId, u64>,
    successful_requests: u64,
    ft_faucet: UnorderedMap<AccountId, FTconfig>,
    blacklist: LookupSet<AccountId>,
    factory_list: LookupSet<AccountId>,
    mod_list: LookupSet<AccountId>,
    vault_contract_id: AccountId,
    min_balance_threshold: Balance,
    request_allowance: Balance,
    request_gap_required: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Stats {
    successful_requests: u64,
    ft_contracts_listed: u64,
    recent_contributions: Vec<(AccountId, U128)>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    RecentReceivers,
    FTFaucet,
    Blacklist,
    FactoryList,
    ModList,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            recent_contributions: Vec::new(),
            recent_receivers: UnorderedMap::new(StorageKey::RecentReceivers),
            ft_faucet: UnorderedMap::new(StorageKey::FTFaucet),
            successful_requests: 0,
            blacklist: LookupSet::new(StorageKey::Blacklist),
            factory_list: LookupSet::new(StorageKey::FactoryList),
            mod_list: LookupSet::new(StorageKey::ModList),
            vault_contract_id: "vault.nonofficial.testnet".parse().unwrap(),
            min_balance_threshold: 5_000_000_000_000_000_000_000_000_000,
            request_allowance: 20_000_000_000_000_000_000_000_000,
            request_gap_required: 3_600_000,
        }
    }
}

#[near_bindgen]
impl Contract {
    // Request NEAR from the faucet
    pub fn request_near(&mut self, receiver_id: AccountId, request_amount: U128) {
        // check if the receiver is in the blacklist
        require!(
            self.blacklist.contains(&env::predecessor_account_id()) == false,
            "Account has been blacklisted!"
        );
        require!(
            request_amount.0 <= self.request_allowance,
            "Withdraw request too large!"
        );

        // remove expired restrictions
        self.remove_expired_restrictions();
        // check if the receiver has requested recently
        self.check_recent_receivers(&receiver_id);

        // make the transfer
        Promise::new(receiver_id.clone()).transfer(request_amount.0);
        // increment the successful requests
        self.successful_requests += 1;
        // check if additional liquidity is needed
        if env::account_balance() < self.min_balance_threshold {
            self.request_additional_liquidity();
        }
    }

    // TODO optimize
    // Remove expired restrictions
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

    // Add account id to the blacklisted (receiver_id)
    pub fn add_to_blacklist(&mut self, account_id: AccountId) {
        assert_self();
        self.blacklist.insert(account_id);
    }

    // Add a list of accounts to the blacklist
    pub fn batch_add_to_blacklist(&mut self, accounts: Vec<AccountId>) {
        assert_self();
        self.blacklist.extend(accounts);
    }

    // Remove account id from the blacklist
    pub fn remove_from_blacklist(&mut self, account_id: AccountId) {
        assert_self();
        self.blacklist.remove(&account_id);
    }

    // Clears the recent receivers map
    pub fn clear_recent_receivers(&mut self) {
        assert_self();
        self.recent_receivers.clear();
    }

    // Contribute to the faucet contract to get in the list of fame
    #[payable]
    pub fn contribute(&mut self) {
        let contributor: AccountId = env::predecessor_account_id();
        let amount: Balance = env::attached_deposit();

        self.recent_contributions.insert(0, (contributor, amount));
        self.recent_contributions.truncate(10);
    }

    // Get stats
    pub fn get_stats(&self) -> Stats {
        Stats {
            successful_requests: self.successful_requests,
            ft_contracts_listed: self.ft_faucet.len() as u64,
            recent_contributions: self.get_recent_contributions(),
        }
    }

    // Get recent contributors
    pub fn get_recent_contributions(&self) -> Vec<(AccountId, U128)> {
        self.recent_contributions
            .iter()
            .map(|(account_id, amount)| (account_id.clone(), U128(*amount)))
            .collect()
    }

    // Request additional liquidity
    fn request_additional_liquidity(&self) {
        vault_contract::ext(self.vault_contract_id.clone()).request_funds();
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

// UNIT TESTS
// Note: #[private] macro doesn't expand in unit tests
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::ONE_NEAR;

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .is_view(is_view)
            .current_account_id("contract.testnet".parse().unwrap());
        builder
    }

    #[test]
    fn request_near() {
        let mut context = get_context(false);
        context.predecessor_account_id(accounts(0));

        testing_env!(context
            .account_balance(ONE_NEAR * 100)
            .predecessor_account_id(accounts(0))
            .build());

        let mut contract = Contract::default();

        contract.request_near(accounts(0), U128(ONE_NEAR * 20));

        assert_eq!(contract.successful_requests, 1);
        assert!(contract.recent_receivers.contains_key(&accounts(0)));
        assert_eq!(env::account_balance(), 80 * ONE_NEAR);
    }

    #[test]
    #[should_panic]
    fn panics_add_to_blocklist() {
        let mut contract = Contract::default();
        contract.add_to_blacklist(accounts(0));
    }

    #[test]
    fn add_to_blocklist() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        contract.add_to_blacklist(accounts(0));

        // Alice shall be in the blacklist
        assert!(contract.blacklist.contains(&accounts(0)));
    }

    #[test]
    #[should_panic]
    fn panics_batch_add_to_blacklist() {
        let mut contract = Contract::default();
        contract.batch_add_to_blacklist(vec![accounts(0), accounts(1)]);
    }

    #[test]
    fn batch_add_to_blacklist() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        contract.batch_add_to_blacklist(vec![accounts(0), accounts(1)]);

        // Alice and Bob shall be in the blocklist
        assert!(contract.blacklist.contains(&accounts(0)));
        assert!(contract.blacklist.contains(&accounts(1)));
    }

    #[test]
    #[should_panic]
    fn panics_remove_from_blacklist() {
        let mut contract = Contract::default();

        contract.add_to_blacklist(accounts(0));
        contract.remove_from_blacklist(accounts(0));

        // Alice shall not be in the blocklist
        assert!(!contract.blacklist.contains(&accounts(0)));
    }

    #[test]
    #[should_panic]
    fn panics_clear_recent_receivers() {
        let mut contract = Contract::default();
        contract.clear_recent_receivers();
    }

    #[test]
    fn clear_recent_receivers() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        contract.recent_receivers.insert(accounts(0), 0);
        contract.recent_receivers.insert(accounts(1), 0);
        contract.clear_recent_receivers();

        // Alice and Bob shall not be in the recent receivers
        assert!(contract.recent_contributions.is_empty());
    }

    #[test]
    fn contribute() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .account_balance(ONE_NEAR)
            .predecessor_account_id(accounts(0))
            .attached_deposit(ONE_NEAR)
            .build());

        contract.contribute();
        // one near initial + one near contribution
        assert_eq!(env::account_balance(), 2 * ONE_NEAR);
    }

    #[test]
    fn get_recent_contributions() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        // alice context
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(10)
            .build());
        contract.contribute();

        assert_eq!(
            (accounts(0), U128(10)),
            contract.get_recent_contributions()[0]
        );

        // bobs context
        testing_env!(context
            .predecessor_account_id(accounts(1))
            .attached_deposit(11)
            .build());
        contract.contribute();

        assert_eq!(
            (accounts(1), U128(11)),
            contract.get_recent_contributions()[0]
        );

        assert_eq!(
            vec![(accounts(1), U128(11)), (accounts(0), U128(10)),],
            contract.get_recent_contributions()
        )
    }

    // ###
    // FT related tests
    // ###

    #[test]
    #[should_panic]
    fn panics_add_factory() {
        let mut contract = Contract::default();
        contract.add_factory(accounts(0));
    }

    #[test]
    fn add_factory() {
        let mut context = get_context(false);
        let mut contract = Contract::default();

        testing_env!(context
            .predecessor_account_id("contract.testnet".parse().unwrap())
            .build());

        contract.add_factory(accounts(0));

        assert!(contract.factory_list.contains(&accounts(0)));
    }

    #[test]
    #[should_panic]
    fn panics_ft_list_from_factory() {
        let mut contract = Contract::default();

        contract.ft_list_from_factory(U128(0), U128(100), {
            FungibleTokenMetadata {
                spec: "ft-1.0.0".to_string(),
                name: "AwesomeToken".to_string(),
                symbol: "aWT".to_string(),
                decimals: 9,
                icon: None,
                reference: None,
                reference_hash: None,
            }
        });
    }
}
