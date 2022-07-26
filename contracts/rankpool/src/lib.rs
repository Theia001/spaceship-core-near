use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet, LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue, Gas, ext_contract, Promise, PromiseResult, log, serde_json
};

mod view;
mod owner;
mod utils;
mod events;

pub use crate::utils::*;
pub use crate::events::*;

pub type PriceType = u128;
pub type TimeStampSec = u64;


// reward basic
pub const RATE_DENOMINATOR: u32 = 1000;

pub const ONE_WEEK_IN_SEC: u64 = 7*24*60*60;

pub type IdxAccId = String;
#[derive(Serialize, Deserialize,Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
struct RewardInfo {
    num: u32,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    winner: String,
    num: u32,
    rate: u32,
    reward: u128,
    claimed: bool,
}

#[derive( Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RoundHistoryVO {
    index: u32,
    settle_reward: u128,
    end: TimeStampSec,
    user: Vec<User>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize,Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Round {
    index: u64,
    reward: u128,
    settle_reward: u128,
    total_num: u32,
    end: TimeStampSec,
    winner: Vec<String>,
    settle: bool,
}

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_TRANSFER: Gas = Gas(10 * TGAS);

#[ext_contract(ext_tokentia)]
pub trait tokentia{
    fn ft_transfer(
        &mut self, 
        receiver_id: AccountId, 
        amount: U128,
        memo: Option<String>
    );
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    UserRound,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    duration_sec: TimeStampSec,  // default: 7*24*60*60  7 days
    token_tia: AccountId,
    boxmall: AccountId,
    shipmarket: AccountId,
    round_history: Vector<Round>,

    user_round: LookupMap<IdxAccId, User>,
    
    balance: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_tia: AccountId, boxmall: AccountId, shipmarket: AccountId ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut this = Contract {
            owner_id,
            round_history: Vector::new(b"v".to_vec()),
            user_round: LookupMap::new( StorageKey::UserRound),
            token_tia,
            boxmall,
            shipmarket,
            duration_sec: ONE_WEEK_IN_SEC,
            balance: 0,
        };
        this.round_history.push(
            &Round{
                index : 0,
                reward : 0,
                settle_reward : 0,
                total_num : 0,
                end : nano_to_sec(env::block_timestamp()) + ONE_WEEK_IN_SEC,
                winner : vec![],
                settle : false
            }
        );
        this
    }
}


#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is msg->{"to": "buyer_id"}.
    /// 
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let predecessor_id = env::predecessor_account_id();

        require!( predecessor_id == self.token_tia , "Invalid contract Id");
        require!( sender_id == self.boxmall || sender_id == self.shipmarket, "Invalid contract Id");

        self.balance += amount.0;

        let reward_info: RewardInfo = serde_json::from_str(&msg).unwrap();

        self.reward(amount.0,reward_info.num);

        Event::Reward{caller_id:&sender_id, amount: &amount, num: reward_info.num}.emit();
        //
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl Contract {
    /* ========== VIEW FUNCTION ========== */
    pub fn get_round_length(&self) -> u64 {
        return self.round_history.len();
    }

    pub fn get_latest_index(&self) -> u64 {
        self.round_history.len() - 1
    }

    pub fn get_latest(&self) -> Round {
        return self.round_history.get(self.round_history.len() - 1).unwrap().clone();
    }

    pub fn get_round_history(&self) -> Vec<Round> {
        //
        let mut length: u64 = 0;
        if self.round_history.len() == 0 {
            length = 0;
        }
        else {
            length = self.round_history.len() - 1;
        }

        let mut history: Vec<Round> = vec![];
        if length == 0 {
            return history;
        }

        for i in 0..length {
            let round_index: u64 = self.round_history.len() - 2 - i;
            history.push(self.round_history.get(round_index).unwrap().clone())
        }
        return history;
        //
    }

    /* ========== CORE FUNCTION ========== */
    pub fn reward(&mut self, amount: u128, num: u32) {
        //IERC20(ssp).safeTransferFrom(msg.sender, address(this), amount);
        let latest_index: u64 = self.get_latest_index();
        let mut round:Round = self.round_history.get(latest_index).unwrap().clone();
        round.reward += amount;
        round.total_num += num;
        self.round_history.replace(latest_index, &round);
     }

    pub fn claim(&mut self, index: u32) {
        let predecessor_id = env::predecessor_account_id();
        let index_acc_id: IdxAccId = format!("{}:{}",index, predecessor_id);

        let mut user:User = self.user_round.get(&index_acc_id).unwrap().clone();//[msg.sender];
        require!(!user.claimed, "already claimed");
        user.claimed = true;
        require!(user.reward > 0, "not enough reward claim");

        ext_tokentia::ft_transfer(
            predecessor_id.clone(),
            U128(user.reward),
            None,
            self.token_tia.clone(),
            1,
            GAS_FOR_TRANSFER
        );
    }
    
    pub fn settle(&mut self, user: Vec<AccountId>, num:Vec<u32>, rate: Vec<u32>) {
        //let predecessor_id = env::predecessor_account_id();
        let latest_index: u64 = self.get_latest_index();
        let mut round: Round = self.round_history.get(latest_index).unwrap().clone();
        require!(round.settle == false, "Round: already settled");
        require!(round.end <= nano_to_sec(env::block_timestamp()), "Round: not end");
        round.settle = true;

        for i in 0..user.len() {
            let user_reward = round.reward*rate[i] as u128/RATE_DENOMINATOR as u128;
            round.winner.push(user[i].to_string().clone());

            let index_acc_id: IdxAccId = format!("{}:{}",latest_index,user[i]);
            self.user_round.insert(&index_acc_id,
                                &User{
                                    winner:user[i].to_string().clone(),
                                    num: num[i],
                                    rate: rate[i],
                                    reward: user_reward,
                                    claimed: false,
                                });

            round.settle_reward = round.settle_reward + user_reward;
        }
        self.round_history.replace(latest_index, &round);

        let addr: Vec<String> = vec![];
        self.round_history.push(&Round{
            index: self.round_history.len(),
            reward: round.reward - round.settle_reward,
            settle_reward: 0,
            total_num: 0,
            end: nano_to_sec(env::block_timestamp()) + self.duration_sec,
            winner: addr,
            settle: false
            });

        Event::Settle{
            caller_id: &env::predecessor_account_id(), 
            index: round.index, 
            reward: &U128(round.reward),
            settle_reward: &U128(round.settle_reward), user, rate
        }.emit();
    }
    /* ========== GOVERNANCE ========== */
    #[payable]
    pub fn set_end(&mut self, index: u64, end: u64) {
        assert_one_yocto();
        assert_eq!( env::predecessor_account_id(), self.owner_id, "rankpool: Owner only" );

        let mut round: Round = self.round_history.get(index).unwrap().clone();
        round.end = end;
        self.round_history.replace(index, &round);

    }

    #[payable]
    pub fn set_duration(&mut self, duration_sec: u64) {
        assert_one_yocto();
        assert_eq!( env::predecessor_account_id(), self.owner_id, "rankpool: Owner only" );
        self.duration_sec = duration_sec;
    }
}