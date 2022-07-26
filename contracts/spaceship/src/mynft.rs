use crate::*;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenResolver;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::events::{NftBurn, NftMint, NftTransfer};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, env, ext_contract, require, AccountId, Balance, Gas, PromiseOrValue,
    PromiseResult,
};
use std::collections::HashMap;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(20_000_000_000_000);
const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(35_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

const NO_DEPOSIT: Balance = 0;

#[ext_contract(ext_self)]
trait NFTResolver {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}

#[ext_contract(ext_receiver)]
pub trait NonFungibleTokenReceiver {
    /// Returns true if token should be returned to `sender_id`
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

/// Implementation of the non-fungible token standard.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MyNonFungibleToken {
    // owner of contract
    pub owner_id: AccountId,

    // always required
    pub owner_by_id: LookupMap<TokenId, AccountId>,
    pub supply: Balance,
    pub burned: Balance,

    // required by enumeration extension
    pub tokens_per_owner: Option<LookupMap<AccountId, UnorderedSet<TokenId>>>,
    pub user_count: Balance,
}

impl MyNonFungibleToken {
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            owner_by_id: LookupMap::new(StorageKey::NonFungibleToken),
            supply: 0,
            burned: 0,
            tokens_per_owner: Some(LookupMap::new(StorageKey::Enumeration)),
            user_count: 0,
        }
    }

    /// Transfer token_id from `from` to `to`
    ///
    /// Do not perform any safety checks or do any logging
    pub fn internal_transfer_unguarded(
        &mut self,
        #[allow(clippy::ptr_arg)] token_id: &TokenId,
        from: &AccountId,
        to: &AccountId,
    ) {
        // update owner
        self.owner_by_id.insert(token_id, to);

        // if using Enumeration standard, update old & new owner's token lists
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            // owner_tokens should always exist, so call `unwrap` without guard
            let mut owner_tokens = tokens_per_owner.get(from).unwrap_or_else(|| {
                env::panic_str("Unable to access tokens per owner in unguarded call.")
            });
            owner_tokens.remove(token_id);
            if owner_tokens.is_empty() {
                tokens_per_owner.remove(from);
                self.user_count -= 1;
            } else {
                tokens_per_owner.insert(from, &owner_tokens);
            }

            let mut new_user_flag = false;
            let mut receiver_tokens = tokens_per_owner.get(to).unwrap_or_else(|| {
                new_user_flag = true;
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(to.as_bytes()),
                })
            });
            if new_user_flag {
                self.user_count += 1;
            }
            receiver_tokens.insert(token_id);
            tokens_per_owner.insert(to, &receiver_tokens);
        }
    }

    /// Transfer from current owner to receiver_id, checking that sender is allowed to transfer.
    /// Clear approvals, if approval extension being used.
    /// Return previous owner and approvals.
    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        #[allow(clippy::ptr_arg)] token_id: &TokenId,
        #[allow(unused_variables)] approval_id: Option<u64>,
        memo: Option<String>,
    ) -> AccountId {
        let owner_id = self
            .owner_by_id
            .get(token_id)
            .unwrap_or_else(|| env::panic_str("Token not found"));

        // check if authorized
        if sender_id != &owner_id {
            env::panic_str("Sender not approved");
        }

        require!(
            &owner_id != receiver_id,
            "Current and next owner must differ"
        );

        self.internal_transfer_unguarded(token_id, &owner_id, receiver_id);

        MyNonFungibleToken::emit_transfer(&owner_id, receiver_id, token_id, Some(sender_id), memo);

        // return previous owner
        owner_id
    }

    fn emit_transfer(
        owner_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &str,
        sender_id: Option<&AccountId>,
        memo: Option<String>,
    ) {
        NftTransfer {
            old_owner_id: owner_id,
            new_owner_id: receiver_id,
            token_ids: &[token_id],
            authorized_id: sender_id.filter(|sender_id| *sender_id == owner_id),
            memo: memo.as_deref(),
        }
        .emit();
    }

    /// Mint a new token without checking:
    /// * Whether the caller id is equal to the `owner_id`
    ///
    /// Returns the newly minted token and emits the mint event
    pub fn internal_mint(&mut self, token_id: TokenId, token_owner_id: AccountId) {
        if self.owner_by_id.get(&token_id).is_some() {
            env::panic_str("token_id must be unique");
        }

        let owner_id: AccountId = token_owner_id;

        // Core behavior: every token must have an owner
        self.owner_by_id.insert(&token_id, &owner_id);
        self.supply += 1;

        // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let mut new_user_flag = false;
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                new_user_flag = true;
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });
            if new_user_flag {
                self.user_count += 1;
            }
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        NftMint {
            owner_id: &owner_id,
            token_ids: &[&token_id],
            memo: None,
        }
        .emit();
    }

    /// Burn a token without checking:
    /// * Whether the caller id is equal to the `owner_id`
    /// * emits the mint event
    pub fn internal_burn(&mut self, token_id: &TokenId) {
        // Core behavior: every token must have an owner
        let owner_id = self.owner_by_id.remove(&token_id).unwrap();

        // if using Enumeration standard, update old & new owner's token lists
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            // owner_tokens should always exist, so call `unwrap` without guard
            let mut owner_tokens = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                env::panic_str("Unable to access tokens per owner in unguarded call.")
            });
            owner_tokens.remove(token_id);
            if owner_tokens.is_empty() {
                tokens_per_owner.remove(&owner_id);
                self.user_count -= 1;
            } else {
                tokens_per_owner.insert(&owner_id, &owner_tokens);
            }
        }

        self.burned += 1;
        self.supply -= 1;

        NftBurn {
            owner_id: &owner_id,
            token_ids: &[token_id],
            authorized_id: None,
            memo: None,
        }
        .emit();
    }

    pub fn internal_burned_count(&self) -> Balance {
        self.burned
    }

    pub fn internal_user_count(&self) -> Balance {
        self.user_count
    }
}

impl NonFungibleTokenCore for MyNonFungibleToken {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
    }

    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        require!(
            env::prepaid_gas() > GAS_FOR_NFT_TRANSFER_CALL,
            "More gas is required"
        );
        let sender_id = env::predecessor_account_id();
        let old_owner =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);

        // Initiating receiver's call and the callback
        ext_receiver::nft_on_transfer(
            sender_id.clone(),
            old_owner.clone(),
            token_id.clone(),
            msg.clone(),
            receiver_id.clone(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL,
        )
        .then(ext_self::nft_resolve_transfer(
            old_owner,
            receiver_id.clone(),
            token_id.clone(),
            None,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let owner_id = self.owner_by_id.get(&token_id)?;
        Some(Token {
            token_id,
            owner_id,
            metadata: None,
            approved_account_ids: None,
        })
    }
}

impl NonFungibleTokenResolver for MyNonFungibleToken {
    /// Returns true if token was successfully transferred to `receiver_id`.
    #[allow(unused_variables)]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        //
        // Get whether token should be returned
        let must_revert = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(yes_or_no) = near_sdk::serde_json::from_slice::<bool>(&value) {
                    yes_or_no
                } else {
                    true
                }
            }
            PromiseResult::Failed => true,
        };

        // if call succeeded, return early
        if !must_revert {
            return true;
        }

        // OTHERWISE, try to set owner back to previous_owner_id and restore approved_account_ids

        // Check that receiver didn't already transfer it away or burn it.
        if let Some(current_owner) = self.owner_by_id.get(&token_id) {
            if current_owner != receiver_id {
                // The token is not owned by the receiver anymore. Can't return it.
                return true;
            }
        } else {
            // The token was burned and doesn't exist anymore.
            return true;
        };

        self.internal_transfer_unguarded(&token_id, &receiver_id, &previous_owner_id);

        MyNonFungibleToken::emit_transfer(&receiver_id, &previous_owner_id, &token_id, None, None);
        false
        //
    }
}

impl MyNonFungibleToken {
    /// Helper function used by a enumerations methods
    /// Note: this method is not exposed publicly to end users
    fn enum_get_token(&self, owner_id: AccountId, token_id: TokenId) -> Token {
        Token {
            token_id,
            owner_id,
            metadata: None,
            approved_account_ids: None,
        }
    }
}

impl NonFungibleTokenEnumeration for MyNonFungibleToken {
    fn nft_total_supply(&self) -> U128 {
        self.supply.into()
    }

    /// this interface is not supported
    #[allow(unused_variables)]
    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        vec![]
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        let tokens_per_owner = self.tokens_per_owner.as_ref().unwrap_or_else(|| {
            env::panic_str(
                "Could not find tokens_per_owner when calling a method on the \
                enumeration standard.",
            )
        });
        tokens_per_owner
            .get(&account_id)
            .map(|account_tokens| U128::from(account_tokens.len() as u128))
            .unwrap_or(U128(0))
    }

    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let tokens_per_owner = self.tokens_per_owner.as_ref().unwrap_or_else(|| {
            env::panic_str(
                "Could not find tokens_per_owner when calling a method on the \
                enumeration standard.",
            )
        });
        let token_set = if let Some(token_set) = tokens_per_owner.get(&account_id) {
            token_set
        } else {
            return vec![];
        };
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        require!(limit != 0, "Cannot provide limit of 0.");
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        require!(
            token_set.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        token_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.enum_get_token(account_id.clone(), token_id))
            .collect()
    }
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        require!(
            self.tokens.owner_by_id.get(&token_id) == Some(env::predecessor_account_id()),
            "ERR: Token not found"
        );
        self.update_owner_supply(
            &token_id,
            Some(env::predecessor_account_id()),
            Some(receiver_id.clone()),
        );
        self.tokens
            .nft_transfer(receiver_id, token_id, approval_id, memo)
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        require!(
            self.tokens.owner_by_id.get(&token_id) == Some(env::predecessor_account_id()),
            "ERR: Token not found"
        );
        self.update_owner_supply(
            &token_id,
            Some(env::predecessor_account_id()),
            Some(receiver_id.clone()),
        );
        self.tokens
            .nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens.nft_token(token_id).and_then(|token| {
            Some(Token {
                token_id: token.token_id.clone(),
                owner_id: token.owner_id.clone(),
                metadata: Some(self.gen_metadata(&token.token_id)),
                approved_account_ids: None,
            })
        })
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
    ) -> bool {
        let ret = self.tokens.nft_resolve_transfer(
            previous_owner_id.clone(),
            receiver_id.clone(),
            token_id.clone(),
            approved_account_ids,
        );
        if !ret {
            self.update_owner_supply(
                &token_id,
                Some(receiver_id.clone()),
                Some(previous_owner_id.clone()),
            );
        }
        ret
    }
}

#[near_bindgen]
impl NonFungibleTokenEnumeration for Contract {
    fn nft_total_supply(&self) -> near_sdk::json_types::U128 {
        self.tokens.nft_total_supply()
    }

    fn nft_tokens(
        &self,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        self.tokens.nft_tokens(from_index, limit)
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> near_sdk::json_types::U128 {
        self.tokens.nft_supply_for_owner(account_id)
    }

    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        self.tokens
            .nft_tokens_for_owner(account_id, from_index, limit)
            .into_iter()
            .map(|token| Token {
                token_id: token.token_id.clone(),
                owner_id: token.owner_id.clone(),
                metadata: Some(self.gen_metadata(&token.token_id)),
                approved_account_ids: None,
            })
            .collect()
    }
}
