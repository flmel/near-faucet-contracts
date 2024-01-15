use crate::external::*;
use crate::*;

use near_contract_standards::fungible_token::{
    metadata::FungibleTokenMetadata, receiver::FungibleTokenReceiver,
};
use near_sdk::{
    env, log,
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    Gas, PromiseError, PromiseOrValue, ONE_NEAR, ONE_YOCTO,
};

pub const TGAS: u64 = 1_000_000_000_000;

// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiverMessage {
    List { ft_request_allowance: U128 },
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FTconfig {
    ft_request_allowance: Balance,
    ft_available_balance: Balance,
    ft_metadata: FungibleTokenMetadata,
}

#[near_bindgen]
#[derive(Serialize, BorshSerialize, BorshDeserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FT {
    ft_contract_id: AccountId,
    ft_config: FTconfig,
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    #[allow(unused_variables)] // we don't make use of sender_id
    fn ft_on_transfer(
        &mut self,
        _sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // if the message is ""
        if msg.is_empty() {
            match self.ft_faucet.contains_key(&env::predecessor_account_id()) {
                false => {
                    // Token not listed: refund
                    log!("This FT Contract has not been listed");
                    return PromiseOrValue::Value(amount);
                }
                true => {
                    // Token listed: update faucet balance
                    self.ft_faucet
                        .get_mut(&env::predecessor_account_id())
                        .unwrap()
                        .ft_available_balance += amount.0;

                    return PromiseOrValue::Value(U128(0));
                }
            }
        }

        // If the message is List add it to the ft_faucet HashMap
        let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect("WRONG MSG FORMAT");

        match message {
            TokenReceiverMessage::List {
                ft_request_allowance,
            } => {
                // TODO revaluate GAS attached
                // The message matches we do XCC to get the ft_metadata
                let promise = ft_contract::ext(env::predecessor_account_id())
                    .with_static_gas(Gas(50 * TGAS))
                    .ft_metadata()
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(50 * TGAS))
                            .ft_list_token(
                                env::predecessor_account_id(),
                                ft_request_allowance.0,
                                amount.0,
                            ),
                    );
                PromiseOrValue::Value(U128(0))
            }
        }
    }
}

#[near_bindgen]
impl Contract {
    // List token from a factory contract
    pub fn ft_list_from_factory(
        &mut self,
        ft_request_allowance: U128,
        ft_initial_balance: U128,
        ft_metadata: FungibleTokenMetadata,
    ) {
        require!(
            self.factory_list.contains(&env::signer_account_id()),
            "Only factories can perform this action"
        );

        let ft_request_allowance = ft_request_allowance.0;

        self.ft_faucet.insert(
            env::predecessor_account_id(),
            FTconfig {
                ft_request_allowance,
                ft_available_balance: ft_initial_balance.0,
                ft_metadata,
            },
        );
    }

    // Add a contract to the factory list
    pub fn add_factory(&mut self, factory_id: AccountId) {
        assert_self();
        self.factory_list.insert(factory_id);
    }

    // List new FT in the Faucet
    pub fn ft_list_token(
        &mut self,
        #[callback_result] call_result: Result<FungibleTokenMetadata, PromiseError>,
        ft_contract_id: AccountId,
        ft_request_allowance: Balance,
        ft_available_balance: Balance,
    ) {
        assert_self();
        match call_result {
            Ok(ft_metadata) => {
                // result is Ok store into ft_faucet HashMap
                self.ft_faucet.insert(
                    ft_contract_id,
                    FTconfig {
                        ft_request_allowance,
                        ft_available_balance,
                        ft_metadata,
                    },
                );
                // log Successful message
                log!("Token added Successfully");
            }
            // log Error message
            Err(err) => log!("{:#?}", err),
        }

        log!("If you made a mistake or want to know more visit https://near-faucet.io/faq");
    }

    // Change allowance
    pub fn ft_change_allowance(&mut self, new_request_allowance: Balance) {
        // check if the FT is listed
        require!(
            self.ft_faucet.contains_key(&env::predecessor_account_id()),
            "This FT Contract has not been listed"
        );

        self.ft_faucet
            .get_mut(&env::predecessor_account_id())
            .unwrap()
            .ft_request_allowance = new_request_allowance;
        log!(
            "The request allowance for this contract has been updated to {}",
            new_request_allowance
        );
    }

    // Remove Token
    // TODO Return the remaining FT to arbitrary account or burn them
    pub fn ft_remove_token(&mut self, confirm: bool) {
        require!(confirm, "Warning, you have to call with confirm argument");

        match self.ft_faucet.remove_entry(&env::predecessor_account_id()) {
            Some((_contract_id, ft_config)) => {
                log!(
                    "Token {} has been removed from the faucet",
                    ft_config.ft_metadata.name
                );
            }
            None => log!("This FT Contract has not been listed"),
        }
    }

    // Get Token FTconfig
    pub fn ft_get_token_config(&self, ft_contract_id: AccountId) -> Option<&FTconfig> {
        match self.ft_faucet.get(&ft_contract_id) {
            Some(ft_config) => Some(ft_config),
            None => {
                log!("This FT Contract has not been listed");
                None
            }
        }
    }

    // List all Tokens
    pub fn ft_list_tokens(&self) -> Vec<FT> {
        self.ft_faucet
            .iter()
            .map(|(k, v)| FT {
                ft_contract_id: k.clone(),
                ft_config: v.clone(),
            })
            .collect::<Vec<FT>>()
    }

    // Request FT
    pub fn ft_request_funds(
        &mut self,
        ft_contract_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) {
        require!(
            self.blacklist.contains(&receiver_id) == false,
            "Account has been blacklisted!".to_owned()
        );
        match self.ft_faucet.get(&ft_contract_id) {
            // ft contract is not listed
            None => {
                log!("This FT Contract has not been listed");
            }

            // ft contract is listed
            Some(ft_contract) => {
                require!(
                    amount.0 <= ft_contract.ft_request_allowance,
                    "Requested amount is higher than the allowance"
                );
                require!(
                    amount.0 <= ft_contract.ft_available_balance,
                    "Requested amount is higher than the available balance of",
                );
                // check if the receiver has requested recently
                self.check_recent_receivers(&receiver_id);

                // storage_deposit_arguments
                let storage_deposit_arguments =
                    //json!({ "account_id": receiver_id, "registration_only": true })
                    json!({ "account_id": receiver_id }).to_string().into_bytes();

                // ft transfer arguments
                let ft_transfer_arguments = json!({ "receiver_id": receiver_id, "amount": amount })
                    .to_string()
                    .into_bytes();

                // TODO revaluate GAS attached
                // register the receiver_id in the FT contract, transfer the funds and update the available FT balance
                Promise::new(ft_contract_id.clone())
                    .function_call(
                        "storage_deposit".to_owned(),
                        storage_deposit_arguments,
                        ONE_NEAR / 10,
                        Gas(5 * TGAS),
                    )
                    .function_call(
                        "ft_transfer".to_owned(),
                        ft_transfer_arguments,
                        ONE_YOCTO,
                        Gas(20 * TGAS),
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .update_ft_balance_and_stats(ft_contract_id, amount),
                    );
            }
        }
    }

    // Update FT balance and stats
    pub fn update_ft_balance_and_stats(
        &mut self,
        ft_contract_id: AccountId,
        amount: U128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        assert_self();
        match call_result {
            // log Error message
            Err(err) => log!("{:#?}", err),
            Ok(_) => {
                self.ft_faucet
                    .get_mut(&ft_contract_id)
                    .unwrap()
                    .ft_available_balance -= amount.0;

                self.successful_requests += 1;
                log!("FT Token balance updated");
            }
        }
    }
}
