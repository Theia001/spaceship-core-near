use std::collections::HashMap;

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, assert_one_yocto, require, AccountId, 
    PanicOnDefault, PromiseOrValue
};
use near_sdk::serde::{Deserialize, Serialize};

mod owner;
mod view;

pub use crate::view::*;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    token: FungibleToken,
    name: String,
    symbol: String,
    icon: Option<String>,
    decimals: u8,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, decimals: u8) -> Self {
        Self {
            owner_id,
            token: FungibleToken::new(b"t".to_vec()),
            name,
            symbol,
            icon: None,
            decimals,
        }
    }

    #[payable]
    pub fn burn(&mut self, amount: U128) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.token.internal_withdraw(&sender_id, amount.into());
    }

    /// Arguments:
    /// - `receiver_ids` - each receivers account ID, an empty string means burn token.
    /// - `amounts` - the amount of tokens to each receiver_id.
    /// - `memo` - a string message that was passed with this transfer, will be recorded as log
    #[payable]
    pub fn batch_transfer(
        &mut self,
        receiver_ids: Vec<String>,
        amounts: Vec<U128>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let receivers: HashMap<_, _> = receiver_ids.iter().zip(amounts.iter()).collect();
        for (receiver, amount) in receivers {
            if amount.0 == 0_u128 {
                continue
            }
            if receiver.is_empty() {
                // burn token
                self.token.internal_withdraw(&sender_id, amount.clone().into());
            } else {
                let receiver_id: AccountId = receiver.parse().expect("ERR_INVALID_RECEIVER_ID");
                self.token.internal_transfer(&sender_id, &receiver_id, amount.clone().into(), memo.clone());
            }   
        }
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            icon: self.icon.clone(),
            reference: None,
            reference_hash: None,
            decimals: self.decimals,
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env};

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(0), String::from("TBD"), String::from("TBD"), 24);
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(1)
            .build());
        contract.mint(1_000_000.into());
        assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000.into());

        testing_env!(context
            .attached_deposit(125 * env::storage_byte_cost())
            .build());
        contract.storage_deposit(Some(accounts(1)), None);
        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.ft_transfer(accounts(1), 1_000.into(), None);
        assert_eq!(contract.ft_balance_of(accounts(1)), 1_000.into());

        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(1))
            .build());
        contract.burn(500.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());

        testing_env!(context
            .attached_deposit(125 * env::storage_byte_cost())
            .predecessor_account_id(accounts(0))
            .build());
        contract.storage_deposit(Some(accounts(2)), None);
        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.batch_transfer(
            vec![accounts(1).to_string(), accounts(2).to_string(), "".to_string()], 
            vec![1_000.into(), 1_000.into(), 1_000.into()], 
            None
        );
        assert_eq!(contract.ft_balance_of(accounts(0)), 996_000.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 1500.into());
        assert_eq!(contract.ft_balance_of(accounts(2)), 1000.into());
        assert_eq!(contract.ft_total_supply(), 998_500.into());
    }
}