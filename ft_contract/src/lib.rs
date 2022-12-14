use near_contract_standards::fungible_token::{
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleToken,
};

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupSet},
    env, ext_contract,
    json_types::U128,
    near_bindgen, require, AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise,
    PromiseOrValue,
};

mod storage;
// Error message
const ERR_MOD_REQUIRED: &str = "Only mods can perform this action";

#[ext_contract(faucet_contract)]
trait FaucetContract {
    fn ft_remove_token(confirm: bool);
    fn ft_list_from_factory(
        ft_request_allowance: U128,
        ft_initial_balance: U128,
        ft_metadata: FungibleTokenMetadata,
    );
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    mod_list: LookupSet<AccountId>,
    registered_accounts: u64,
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
            registered_accounts: 1, // owner_id is registered by default
        };
        // enlist the factory account as a mod
        this.mod_list.insert(&env::predecessor_account_id());
        // register the owner and mint the total supply
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());

        // emit the mint event
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();

        this
    }

    // Add account to the mod list
    pub fn add_mod(&mut self, account_id: AccountId) {
        require!(
            self.mod_list.contains(&env::predecessor_account_id()),
            ERR_MOD_REQUIRED
        );
        self.mod_list.insert(&account_id);
    }

    // Get registered accounts count
    pub fn get_registered_accounts(&self) -> u64 {
        self.registered_accounts
    }

    // Add the token to the faucet contract
    pub fn list_on_faucet(
        &mut self,
        faucet_account_id: AccountId,
        ft_request_allowance: U128,
        ft_initial_balance: U128,
    ) {
        require!(
            self.mod_list.contains(&env::predecessor_account_id()),
            ERR_MOD_REQUIRED
        );
        require!(
            ft_request_allowance.0 < ft_initial_balance.0,
            "Set the request allowance to be less than the initial balance"
        );

        // check if the token is already listed
        match self.token.accounts.contains_key(&faucet_account_id) {
            true => panic!("Token is already listed on the faucet"),
            false => {
                // register the faucet account
                self.token.internal_register_account(&faucet_account_id);
                self.registered_accounts += 1;
                //  increase the supply and deposit it to the faucet account
                self.token
                    .internal_deposit(&faucet_account_id, ft_initial_balance.0);
            }
        };

        // emit the mint event
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &faucet_account_id,
            amount: &ft_initial_balance,
            memo: Some("Tokens minted for the faucet contract"),
        }
        .emit();
        // call the faucet contract to add the token
        faucet_contract::ext(faucet_account_id)
            .with_static_gas(Gas(10 * 10u64.pow(12)))
            .ft_list_from_factory(
                ft_request_allowance,
                ft_initial_balance,
                self.metadata.get().unwrap(),
            );
    }

    // Remove the token from the faucet contract
    pub fn remove_from_faucet(&mut self, faucet_account_id: AccountId, confirm: bool) {
        require!(
            self.mod_list.contains(&env::predecessor_account_id()),
            ERR_MOD_REQUIRED
        );
        require!(confirm, "You must confirm the action");
        // remove the contract from the faucet
        faucet_contract::ext(faucet_account_id)
            .with_static_gas(Gas(5 * 10u64.pow(12)))
            .ft_remove_token(confirm);
    }

    // Delete the contract account
    // Self destruct !!!
    pub fn delete_contract_account(&mut self) {
        require!(
            self.mod_list.contains(&env::predecessor_account_id()),
            ERR_MOD_REQUIRED
        );
        Promise::new(env::current_account_id()).delete_account(env::predecessor_account_id());
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id("contract.testnet".parse().unwrap())
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());

        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 24,
        };

        let contract = Contract::new(accounts(1).into(), TOTAL_SUPPLY.into(), metadata);

        testing_env!(context.is_view(true).build());
        // contract shall be deployed
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
        // metadata should be correct
        assert_eq!(contract.ft_metadata().decimals, 24);
        assert_eq!(contract.ft_metadata().name, "Test Token");
        // predecessor account shall be registered as a mod
        assert!(contract.mod_list.contains(&accounts(1)));
        // count of registered accounts shall be 1 (owner)
        assert_eq!(contract.registered_accounts, 1);
    }

    // #[test]
    // #[should_panic(expected = "The contract is not initialized")]
    // fn default() {
    //     let context = get_context(accounts(1));
    //     testing_env!(context.build());
    //     let _contract = Contract::default();
    // }

    // #[test]
    // fn transfer() {
    //     let mut context = get_context(accounts(2));
    //     testing_env!(context.build());
    //     let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
    //     testing_env!(context
    //         .storage_usage(env::storage_usage())
    //         .attached_deposit(contract.storage_balance_bounds().min.into())
    //         .predecessor_account_id(accounts(1))
    //         .build());
    //     // Paying for account registration, aka storage deposit
    //     contract.storage_deposit(None, None);

    //     testing_env!(context
    //         .storage_usage(env::storage_usage())
    //         .attached_deposit(1)
    //         .predecessor_account_id(accounts(2))
    //         .build());
    //     let transfer_amount = TOTAL_SUPPLY / 3;
    //     contract.ft_transfer(accounts(1), transfer_amount.into(), None);

    //     testing_env!(context
    //         .storage_usage(env::storage_usage())
    //         .account_balance(env::account_balance())
    //         .is_view(true)
    //         .attached_deposit(0)
    //         .build());
    //     assert_eq!(
    //         contract.ft_balance_of(accounts(2)).0,
    //         (TOTAL_SUPPLY - transfer_amount)
    //     );
    //     assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    // }

    // // ========================
    // #[test]
    // fn get_registered_accounts() {
    //     let mut context = get_context(accounts(2));
    //     let mut contract = Contract::default();

    //     contract.ft_transfer(accounts(1), transfer_amount.into(), None);
    // }

    #[test]
    #[should_panic]
    fn panics_add_mod() {
        let mut contract = Contract::default();
        contract.add_mod(accounts(1));
    }

    #[test]
    fn add_mod() {
        let mut context = get_context(accounts(2));
        let mut contract = Contract::default();

        contract.add_mod(accounts(3));
        assert!(contract.mod_list.contains(&accounts(3)));
    }
}
