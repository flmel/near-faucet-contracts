use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::Vector,
    env, ext_contract,
    json_types::U128,
    near_bindgen, AccountId, Promise,
};

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

const CODE: &[u8] = include_bytes!("./ft.wasm");

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    contracts: Vector<AccountId>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            contracts: Vector::new(b'c'),
        }
    }
}

// Interface for cross-contract FT calls
#[ext_contract(ft_contract)]
trait FtContract {
    fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata);
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn create_contract(
        &mut self,
        sub_account_id: AccountId,
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Promise {
        Promise::new(sub_account_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(3_000_000_000_000_000_000_000_000) // 3e24yN, 3N
            .deploy_contract(CODE.to_vec())
            .then(ft_contract::ext(sub_account_id).new(owner_id, total_supply, metadata))
    }

    pub fn num_contracts(&self) -> u64 {
        self.contracts.len() as u64
    }

    // Private
    #[private]
    pub fn add_contract(&mut self, sub_account_id: AccountId) {
        self.contracts.push(&sub_account_id);
    }
}
