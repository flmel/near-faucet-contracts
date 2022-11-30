use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise,
    PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    mod_list: LookupSet<AccountId>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
    ModList,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            mod_list: LookupSet::new(StorageKey::ModList),
        };
        // Enlist the factory contract as a mod
        this.mod_list.insert(&env::predecessor_account_id());
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());

        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
    // Add account to the mod list
    pub fn add_mod(&mut self, account_id: AccountId) {
        require!(
            self.mod_list.contains(&env::predecessor_account_id()),
            "Only mods can add mods"
        );
        self.mod_list.insert(&account_id);
    }
    // Delete the contract account (self destruct!!!)
    pub fn delete_contract_account(&mut self) {
        require!(
            self.mod_list.contains(&env::predecessor_account_id()),
            "Only mods can delete the contract"
        );
        Promise::new(env::current_account_id()).delete_account(env::predecessor_account_id());
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

// #[cfg(all(test, not(target_arch = "wasm32")))]
// mod tests {
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::{testing_env, Balance};

//     use super::*;

//     const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

//     fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .predecessor_account_id(predecessor_account_id);
//         builder
//     }

//     #[test]
//     fn test_new() {
//         let mut context = get_context(accounts(1));
//         testing_env!(context.build());
//         let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
//         testing_env!(context.is_view(true).build());
//         assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
//         assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
//     }

//     #[test]
//     #[should_panic(expected = "The contract is not initialized")]
//     fn test_default() {
//         let context = get_context(accounts(1));
//         testing_env!(context.build());
//         let _contract = Contract::default();
//     }

//     #[test]
//     fn test_transfer() {
//         let mut context = get_context(accounts(2));
//         testing_env!(context.build());
//         let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(contract.storage_balance_bounds().min.into())
//             .predecessor_account_id(accounts(1))
//             .build());
//         // Paying for account registration, aka storage deposit
//         contract.storage_deposit(None, None);

//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .attached_deposit(1)
//             .predecessor_account_id(accounts(2))
//             .build());
//         let transfer_amount = TOTAL_SUPPLY / 3;
//         contract.ft_transfer(accounts(1), transfer_amount.into(), None);

//         testing_env!(context
//             .storage_usage(env::storage_usage())
//             .account_balance(env::account_balance())
//             .is_view(true)
//             .attached_deposit(0)
//             .build());
//         assert_eq!(
//             contract.ft_balance_of(accounts(2)).0,
//             (TOTAL_SUPPLY - transfer_amount)
//         );
//         assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
//     }
// }
