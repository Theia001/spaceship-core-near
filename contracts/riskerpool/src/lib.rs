use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet, LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue, Gas, ext_contract, Promise, PromiseResult, log, serde_json
};
use std::cmp;

mod view;
mod owner;
mod utils;

pub use crate::utils::*;

pub type TimeStampSec = u64;

// reward basic
pub const RATE_DENOMINATOR: u8 = 100;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
struct BuyInfo {
    buyer_id: AccountId,  // AccountId
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Round {
    index: u32,
    current_risker: String,
    winner: String,
    reward: u128,
    end: TimeStampSec,
}

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_TRANSFER: Gas = Gas(10 * TGAS);
pub const GAS_FOR_RESOLVE: Gas = Gas(10 * TGAS);
pub const DEFAULT_REWARD_RATE: u8 = 80;
pub const ONE_DAY_IN_SEC: u64 = 24 * 60 * 40;
pub const SHORT_DURATION_IN_SEC: u64 = 3 * 40;

#[ext_contract(ext_tokentia)]
pub trait tokentia{
    fn batch_transfer(
        &mut self,
        receiver_ids: Vec<String>,
        amounts: Vec<U128>,
        memo: Option<String>,
    );
}

#[ext_contract(ext_self)]
pub trait Resolver {
    fn resolve_cross_call(&mut self) -> U128;
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    TokensPerOwner { account_hash: Vec<u8> },
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,

    token_tia: AccountId,
    boxmall: AccountId,
    
    round_history: Vector<Round>,

    balance: u128,
    
    reward_rate: u8,        // default: 25,
    duration_sec: u64,      // default: 24*60*60 24 hours
    shot_duration_sec: u64, // default: 3*60 3 minutes
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_tia: AccountId, boxmall: AccountId ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id,
            round_history: Vector::new(b"v".to_vec()),
            token_tia,
            boxmall,
            balance: 0,
            reward_rate: DEFAULT_REWARD_RATE,
            duration_sec: ONE_DAY_IN_SEC,
            shot_duration_sec: SHORT_DURATION_IN_SEC,
        }
    }
}


#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is msg->{"to": "buyer_id"}.
    /// BML-00-01
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let predecessor_id = env::predecessor_account_id();

        self.balance += amount.0;

        if sender_id == self.boxmall {
            let buyer: BuyInfo = serde_json::from_str(&msg).unwrap();
            self.shot(buyer.buyer_id, amount.0);
        }

        //
        PromiseOrValue::Value(U128(0))
    }


}

#[near_bindgen]
impl Contract {

    #[private]
    pub fn resolve_cross_call(
        &mut self,
    ) -> U128 {
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(bal) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    log!("Get return value, balance: {:#?}", bal.clone());
                    if self.balance != bal.0 {
                        self.balance = bal.0;
                    }
                    bal
                } else {
                    U128(0)
                }
            }
            PromiseResult::Failed => U128(0),
        }
    }

    /* ========== VIEW FUNCTION ========== */
    pub fn get_balance(&self) -> u128 {
        self.balance
    }

    pub fn sync_balance(&self) {
        ext_fungible_token::ft_balance_of(
            self.owner_id.clone(),
            self.token_tia.clone(),
            0,
            GAS_FOR_TRANSFER
        ).then(ext_self::resolve_cross_call(
            env::current_account_id(),
            0,
            GAS_FOR_RESOLVE,
        ));
    }

    pub fn get_round_length(&self) -> u64 {
        return self.round_history.len();
    }

    pub fn get_latest_index(&self) -> u64 {
        self.round_history.len() - 1
    }

    pub fn get_latest(&self) -> Round {
        let idx = self.round_history.len() - 1;
        return self.round_history.get(idx).unwrap().clone();
    }

    pub fn get_round_history(&self) -> Vec<Round> {
        self.round_history.to_vec()
    }

    /* ========== CORE FUNCTION ========== */
    pub fn shot(&mut self, to: AccountId, amount: u128) {
        self.check_first(to.clone());

        let last_index = self.get_latest_index();

        let mut round: Round = self.round_history.get(last_index).unwrap().clone();
        if round.end <= nano_to_sec(env::block_timestamp()) {
            // reward current round
            round.winner = round.current_risker;
            round.reward = (self.get_balance()-amount)*self.reward_rate as u128/RATE_DENOMINATOR as u128;

            ext_fungible_token::ft_transfer(
                AccountId::new_unchecked(round.winner),
                U128(round.reward),
                None,
                self.token_tia.clone(),
                1,
                GAS_FOR_TRANSFER
            );
            //
            self.balance -= round.reward;

            // next round
            self.round_history.push(&Round{
                index: self.round_history.len() as u32,
                current_risker : to.to_string(),
                winner : "".to_string(),
                reward : 0,
                end : nano_to_sec(env::block_timestamp()) + self.duration_sec
                });
            return;
        }

        // update currentRisker, reward
        round.end = cmp::min(round.end + self.shot_duration_sec, nano_to_sec(env::block_timestamp())+self.duration_sec);
        round.current_risker = to.to_string().clone();
        self.round_history.replace(last_index as u64,&round);
    }
    
    
    /* ========== GOVERNANCE ========== */
    #[payable]
    pub fn set_reward_rate(&mut self, reward_rate: u8) {
        assert_one_yocto();
        assert_eq!( env::predecessor_account_id(), self.owner_id, "riskerpool: Owner only" );
        self.reward_rate = reward_rate;
    }

    #[payable]
    pub fn set_duration(&mut self, duration_sec: u64) {
        assert_one_yocto();
        assert_eq!( env::predecessor_account_id(), self.owner_id, "riskerpool: Owner only" );
        self.duration_sec = duration_sec;
    }

    #[payable]
    pub fn set_shot_duration(&mut self, duration_sec: u64) {
        assert_one_yocto();
        assert_eq!( env::predecessor_account_id(), self.owner_id, "riskerpool: Owner only" );
        self.shot_duration_sec = duration_sec;
    }
    
    fn check_first(&mut self, to: AccountId) {
        if self.round_history.len() == 0 {
            self.round_history.push(&Round{
                index: self.round_history.len() as u32,
                current_risker : to.to_string(),
                winner : "".to_string(),
                reward : 0,
                end : nano_to_sec(env::block_timestamp())+self.duration_sec
                });
        }
    }
}