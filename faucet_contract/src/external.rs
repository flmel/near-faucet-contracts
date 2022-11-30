use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{ext_contract, json_types::U128, AccountId, Balance, PromiseOrValue};

// Interface of this contract, for callbacks
#[ext_contract(this_contract)]
trait Callbacks {
    fn ft_add_token(
        &mut self,
        ft_account_id: AccountId,
        ft_allowance: Balance,
        ft_available: Balance,
    ) -> PromiseOrValue<U128>;
}

// Interface for cross-contract FT calls
#[ext_contract(ft_contract)]
trait FtContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

// Interface for cross-contract Vault calls
#[ext_contract(vault_contract)]
trait VaultContract {
    fn request_funds(&mut self);
}
