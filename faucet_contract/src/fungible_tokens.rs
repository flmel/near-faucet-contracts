use crate::external::*;
use crate::*;

use near_contract_standards::fungible_token::{
    metadata::FungibleTokenMetadata, receiver::FungibleTokenReceiver,
};
use near_sdk::{
    log,
    serde::{Deserialize, Serialize},
    serde_json, Gas, PromiseError, PromiseOrValue, ONE_YOCTO,
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
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FTconfig {
    ft_request_allowance: Balance,
    ft_available_balance: Balance,
    ft_metadata: FungibleTokenMetadata,
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    #[allow(unused_variables)] // we don't make use of sender_id
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // if the message is ""
        if msg.is_empty() {
            match self.ft_faucet.contains_key(&env::predecessor_account_id()) {
                false => {
                    // Token not listed: refund
                    log!("This FT Contract has not been listed yet");
                    return PromiseOrValue::Value(amount);
                }
                true => {
                    // Token listed: update
                    self.ft_faucet
                        .get_mut(&env::predecessor_account_id())
                        .unwrap()
                        .ft_available_balance += amount.0;
                    log!("This FT Contract has been updated");
                    return PromiseOrValue::Value(U128(0));
                }
            }
        }

        // if the message is List add it to the ft_faucet HashMap
        let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect("WRONG MSG FORMAT");

        match message {
            TokenReceiverMessage::List {
                ft_request_allowance,
            } => {
                // TODO maybe assert predecessor = signer
                // TODO case when FT contract does not implement ft_metadata
                // TODO revaluate GAS attached

                // The message matches we do XCC to get the ft_metadata
                let promise = ft_contract::ext(env::predecessor_account_id())
                    .with_static_gas(Gas(50 * TGAS))
                    .ft_metadata()
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(50 * TGAS))
                            .ft_add_token(
                                env::predecessor_account_id(),
                                ft_request_allowance.0,
                                amount.0,
                            ),
                    );
                PromiseOrValue::from(promise)
            }
        }
    }
}

#[near_bindgen]
impl Contract {
    // List new FT in the Faucet
    #[private]
    pub fn ft_add_token(
        &mut self,
        #[callback_result] call_result: Result<FungibleTokenMetadata, PromiseError>,
        ft_account_id: AccountId,
        ft_request_allowance: Balance,
        ft_available_balance: Balance,
    ) -> PromiseOrValue<U128> {
        match call_result {
            Ok(ft_metadata) => {
                // Result is Ok store into ft_faucet HashMap
                self.ft_faucet.insert(
                    ft_account_id,
                    FTconfig {
                        ft_request_allowance,
                        ft_available_balance,
                        ft_metadata,
                    },
                );
                // Log Successful message
                log!("Token added Successfully");
            }
            // Log Error message
            Err(err) => log!("{:#?}", err),
        }
        // TODO add proper docs URL
        log!("If you made a mistake or want to know more visit URL_HERE");

        PromiseOrValue::Value(U128(0))
    }

    // Change allowance
    pub fn ft_change_allowance(&mut self, new_request_allowance: Balance) {
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
    // TODO Return the remaining FT
    pub fn ft_remove_token(&mut self, confirm: bool) {
        require!(confirm, "Warning, you have to call with confirm argument");

        match self.ft_faucet.remove_entry(&env::predecessor_account_id()) {
            Some((_contract_id, ft_config)) => {
                log!(
                    "Token {} has been removed from the faucet",
                    ft_config.ft_metadata.name
                );
            }
            None => log!("This token does not exist"),
        }
    }

    // Get Token FTconfig
    pub fn ft_get_token_info(&self, ft_contract_id: AccountId) -> Option<&FTconfig> {
        match self.ft_faucet.get(&ft_contract_id) {
            Some(ft_config) => Some(ft_config),
            None => {
                log!("This token does not exist");
                None
            }
        }
    }

    // List all Tokens
    pub fn ft_list_tokens(&self) -> Vec<(&AccountId, &FTconfig)> {
        self.ft_faucet.iter().collect()
    }

    // Request FT
    pub fn ft_request_funds(
        &mut self,
        ft_contract_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<U128> {
        match self.ft_faucet.get(&ft_contract_id) {
            Some(ft_contract) => {
                require!(
                    amount.0 <= ft_contract.ft_request_allowance,
                    "Requested amount is higher than the allowance"
                );
                require!(
                    amount.0 <= ft_contract.ft_available_balance,
                    "Requested amount is higher than the available balance of",
                );
                require!(
                    self.blacklist.contains(&env::predecessor_account_id()) == false,
                    "Account has been blacklisted!"
                );
                // TODO Check/Pay the user storage_deposit
                /* TODO Check for recent_receivers
                   this would require design decisions on how to handle multiple requests be in in the main recent receivers or
                   in a separate list of recent receivers for each token
                */

                // Conditions are met we can transfer the funds
                let promise = ft_contract::ext(ft_contract_id)
                    .with_static_gas(Gas(50 * TGAS))
                    .with_attached_deposit(ONE_YOCTO)
                    .ft_transfer(receiver_id, amount, None);
                return PromiseOrValue::from(promise);
            }
            None => {
                log!("This token does not exist");
                PromiseOrValue::Value(U128(0))
            }
        }
    }
}
