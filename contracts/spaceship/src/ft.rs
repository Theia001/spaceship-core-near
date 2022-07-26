use crate::*;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn burn_eng(&mut self, amount: U128) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.eng.internal_withdraw(&sender_id, amount.into());
    }

    #[payable]
    pub fn burn_eng_for_user(&mut self, user: AccountId, amount: U128) {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        require!(
            predecessor_account_id == self.shipmarket_id
                || predecessor_account_id == self.luckpool_id,
            "Invalid contract id"
        );
        self.eng.internal_withdraw(&user, amount.into());
    }

    /// Arguments:
    /// - `receiver_ids` - each receivers account ID, an empty string means burn token.
    /// - `amounts` - the amount of tokens to each receiver_id.
    /// - `memo` - a string message that was passed with this transfer, will be recorded as log
    #[payable]
    pub fn batch_eng_transfer(
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
                continue;
            }
            if receiver.is_empty() {
                // burn eng token
                self.eng
                    .internal_withdraw(&sender_id, amount.clone().into());
            } else {
                let receiver_id: AccountId = receiver.parse().expect("ERR_INVALID_RECEIVER_ID");
                self.eng.internal_transfer(
                    &sender_id,
                    &receiver_id,
                    amount.clone().into(),
                    memo.clone(),
                );
            }
        }
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Engine".to_string(),
            symbol: "ENG".to_string(),
            icon: self.eng_icon.clone(),
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env};

    use super::*;

    #[test]
    fn test_basic_ft() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Contract::new(
            accounts(0),
            accounts(2),
            accounts(3),
            accounts(4),
            accounts(5),
            "luckpool".parse().unwrap(),
        );
        testing_env!(context
            .predecessor_account_id(accounts(0))
            .attached_deposit(1)
            .build());
        contract.mint_eng(1_000_000.into());
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
        contract.burn_eng(500.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());

        assert_eq!(contract.ft_balance_of(accounts(0)), 999_000.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());
        assert_eq!(contract.ft_total_supply(), 999_500.into());
    }
}
