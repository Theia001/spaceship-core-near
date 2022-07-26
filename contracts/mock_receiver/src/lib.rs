use near_contract_standards::non_fungible_token::{TokenId};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;

use near_sdk::{
    env, near_bindgen, require, AccountId, PanicOnDefault,
    PromiseOrValue, log
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    sender_contract_id: AccountId,
 }

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(sender_contract_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Contract {
            sender_contract_id,
         }
    }

    #[allow(unused_variables)]
    pub fn batch_register_ships(&mut self, token_ids: Vec<String>, token_owner_id: AccountId) {
        require!(env::predecessor_account_id()==self.sender_contract_id, "ERR_NOT_ALLOWED");
        log!("[MOCK_RECEIVER] batch_register_ships");
    }
}

#[near_bindgen]
#[allow(unused_variables)]
impl NonFungibleTokenReceiver for Contract {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        require!(env::predecessor_account_id()==self.sender_contract_id, "ERR_NOT_ALLOWED");
        log!("[MOCK_RECEIVER] nft_on_transfer, msg: {}", msg);
        if msg == String::from("return") {
            PromiseOrValue::Value(true)
        } else {
            PromiseOrValue::Value(false)
        }
    }
}

#[near_bindgen]
#[allow(unused_variables)]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        require!(env::predecessor_account_id()==self.sender_contract_id, "ERR_NOT_ALLOWED");
        log!("[MOCK_RECEIVER] ft_on_transfer, msg: {}", msg);
        PromiseOrValue::Value(U128(0))
    }
}