#![allow(unused_variables)]
use near_contract_standards::storage_management::{
    StorageManagement, StorageBalance, StorageBalanceBounds
};
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore,
    metadata::{
        FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
    },
    resolver::FungibleTokenResolver,
    FungibleToken,
};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, assert_one_yocto, AccountId, 
    PanicOnDefault, PromiseOrValue, Promise, Balance
};


#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            token: FungibleToken::new(b"t".to_vec()),
        }
    }

    #[payable]
    pub fn mint(&mut self, amount: U128) {
        assert_one_yocto();
        let user_id = env::predecessor_account_id();
        if self.token.storage_balance_of(user_id.clone()).is_none() {
            self.token.internal_register_account(&user_id);
        }
        self.token.internal_deposit(&user_id, amount.into());
    }


}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        if self.token.storage_balance_of(receiver_id.clone()).is_none() {
            self.token.internal_register_account(&receiver_id);
        }
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        if self.token.storage_balance_of(receiver_id.clone()).is_none() {
            self.token.internal_register_account(&receiver_id);
        }
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, _) =
            self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        used_amount.into()
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        if amount > 0 {
            Promise::new(env::predecessor_account_id()).transfer(amount);
        }
        StorageBalance { total: self.storage_balance_bounds().min, available: 0.into() }
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        env::panic_str(
            "Nothing could be withdraw",
        )
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        env::panic_str(
            "Nothing could be unregister",
        )
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: 0.into(),
            max: Some(0.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        Some(StorageBalance { total: 0.into(), available: 0.into() })
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "USN".to_string(),
            symbol: "usn".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Contract::new();
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(1)
            .build());
        contract.mint(1_000_000.into());
        assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000.into());

        testing_env!(context
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.ft_transfer(accounts(1), 1_000.into(), None);
        assert_eq!(contract.ft_balance_of(accounts(1)), 1_000.into());

    }
}