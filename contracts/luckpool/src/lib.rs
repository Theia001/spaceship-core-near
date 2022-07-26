use std::collections::HashMap;
use near_sdk::{AccountId, Balance, Gas, PromiseOrValue, BorshStorageKey, PanicOnDefault, IntoStorageKey, PromiseResult, CryptoHash, StorageUsage, env,
               assert_one_yocto, ext_contract, near_bindgen, require};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector, LookupMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_sdk::json_types::Base64VecU8;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::events::{NftMint, NftTransfer};
use near_contract_standards::non_fungible_token::{
    hash_account_id, refund_approved_account_ids, refund_deposit_to_account,
};
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenResolver;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

mod view;
mod utils;
mod owner;
mod mynft;
mod events;
pub mod sortition_sum_tree;

use mynft::MyNonFungibleToken;
pub use crate::utils::*;
pub use crate::events::*;

pub const RATE_DENOMINATOR: u8 = 100;
pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_BATCH_TRANSFER: Gas = Gas(25 * TGAS);
pub const GAS_FOR_BATCH_TRANSFER_CALL: Gas = Gas(50 * TGAS);
pub const GAS_FOR_BATCH_MINT_BOX: Gas = Gas(70 * TGAS);
pub const GAS_FOR_TRANSFER: Gas = Gas(25 * TGAS);
pub const GAS_FOR_TRANSFER_ON_CALL: Gas = Gas(45 * TGAS);

pub const DEFAULT_DURATION_IN_SEC: u64 = 7 * 24 * 60 * 60;

#[ext_contract(ext_tokentia)]
pub trait TokenTia {
    fn batch_transfer(
        &mut self,
        receiver_ids: Vec<String>,
        amounts: Vec<U128>,
        memo: Option<String>,
    );
}

#[ext_contract(ext_spaceship)]
pub trait Spaceship {
    fn burn_eng_for_user(&mut self, user: AccountId, amount: U128);
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    NonFungibleTokenKey,
    TokenMetadata,
    Enumeration,
    Approval,
    TokensPerOwner { account_hash: Vec<u8> },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Round {
    index: u64,
    #[serde(with = "u128_dec_format")]
    settle_reward: u128,
    #[serde(with = "u128_dec_format")]
    total_amount: u128,
    end: u64,
    winner1: AccountId,
    winner2: AccountId,
    winner3: AccountId,
    #[serde(with = "u128_dec_format")]
    reward1: u128,
    #[serde(with = "u128_dec_format")]
    reward2: u128,
    #[serde(with = "u128_dec_format")]
    reward3: u128,
    settle: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    amount: u32,
    total_amount: u32,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,

    token_tia: AccountId,
    spaceship: AccountId,

    round_history: Vector<Round>,
    duration: u64,

    id: u64,
    shift: usize,
    reward: u128,
    settle_reward: u128,
    claimed_reward: u128,
    reward_rate1: u8,
    reward_rate2: u8,
    reward_rate3: u8,

    tokens: MyNonFungibleToken,

    balance: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_tia: AccountId, spaceship: AccountId, name: String, symbol: String) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id: owner_id.clone(),
            token_tia,
            spaceship,

            round_history: Vector::new(b"l".to_vec()),
            duration: DEFAULT_DURATION_IN_SEC, // 七天，单位秒

            id: 0,
            shift: 0,
            reward: 0,
            settle_reward: 0,
            claimed_reward: 0,

            reward_rate1: 40,
            reward_rate2: 24,
            reward_rate3: 16,

            balance: 0,

            tokens: MyNonFungibleToken::new(
                StorageKey::NonFungibleTokenKey,
                owner_id.clone(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
                name,
                symbol),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn create_round(&mut self) {
        let len = self.round_history.len() as u64;
        let addr: AccountId = AccountId::new_unchecked("00".to_string());

        self.tokens.create_new_tree();

        self.round_history.push(&Round {
            index: len,
            settle_reward: 0,
            total_amount: 0,
            end: nano_to_sec(env::block_timestamp()) + self.duration,
            winner1: addr.clone(),
            winner2: addr.clone(),
            winner3: addr.clone(),
            reward1: 0,
            reward2: 0,
            reward3: 0,
            settle: false,
        })
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env};

    use super::*;

    #[test]
    fn create_round_test() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());

        let mut contract = Contract::new(
            accounts(0),
            accounts(1),
            accounts(2),
            String::from("name"),
            String::from("symbol"));

        contract.create_round();

        let length = contract.get_round_length();
        assert_eq!(length, 1);

        let list = contract.get_round_history();
        list.iter().for_each(|item| {
            println!("{:?}", item);
        });

        // let metadata = contract.get_metadata();

        let latest_index = contract.get_latest_index();
        assert_eq!(latest_index, 0);

        let round = contract.get_round_info(0);
        println!("round_info: {:?}", round);

        // 创建第二轮
        println!("--------------------第二轮分界线--------------------");
        contract.create_round();

        let length = contract.get_round_length();
        assert_eq!(length, 2);

        let list = contract.get_round_history();
        list.iter().for_each(|item| {
            println!("{:?}", item);
        });

        // let metadata = contract.get_metadata();

        let latest_index = contract.get_latest_index();
        assert_eq!(latest_index, 1);

        let round = contract.get_round_info(0);
        println!("round_info 0: {:?}", round);
        let round = contract.get_round_info(1);
        println!("round_info 1: {:?}", round);

    }

    #[test]
    fn stake_and_settle() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        context.predecessor_account_id(accounts(5));
        context.storage_usage(0);
        testing_env!(context
            .attached_deposit(125 * env::storage_byte_cost())
            .build());

        let mut contract = Contract::new(
            AccountId::new_unchecked("bob.near".to_string()),
            accounts(1),
            accounts(2),
            String::from("name"),
            String::from("symbol"));


        contract.create_round();

        let list = contract.get_round_history();
        list.iter().for_each(|item| {
            println!("{:?}", item);
        });

        let latest_index = contract.get_latest_index();
        testing_env!(context
            .attached_deposit(6000000000000000000000)
            .predecessor_account_id(AccountId::new_unchecked("bob.near".to_string()))
            .build());
        contract.stake(latest_index, U128(1));
        contract.settle();

        assert_eq!(list.len(), 1);

        let list = contract.get_round_history();
        list.iter().for_each(|item| {
            println!("{:?}", item);
        });

    }
}
