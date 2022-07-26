use near_contract_standards::non_fungible_token::{TokenId};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue, Gas,  PromiseResult, log, Balance, ext_contract
};
use std::cmp;

mod view;
mod owner;
mod utils;
mod events;

pub use crate::utils::*;
pub use crate::events::*;

pub type RoundIdAcc = String;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_TRANSFER: Gas = Gas(25 * TGAS);
pub const GAS_FOR_TRANSFER_ON_CALL: Gas = Gas(45 * TGAS);

pub const YOCTO18: u128 = 1_000_000_000_000_000_000;

pub const TYPE_S: u8 = 5;
pub const TYPE_A: u8 = 4;
pub const TYPE_B: u8 = 3;
pub const TYPE_C: u8 = 2;
pub const TYPE_D: u8 = 1;

pub const TYPE_A_MAX: u8 = 4;
pub const TYPE_B_MAX: u8 = 8;
pub const TYPE_C_MAX: u8 = 16;
pub const TYPE_D_MAX: u8 = 32;

#[derive( BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct Round {
    pioneer: Vec<AccountId>,
    round_reward: u128,
    total_pioneer_token_amount: u128,
    reward_time: u64,
    avg_reward: u128,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct RoundVO {
    pub pioneer: Vec<AccountId>,
    pub round_reward: U128,
    pub index: u32,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Pioneer {
    pub pioneer: String,
    pub pioneer_token_amount: u128,
    pub claimed: bool,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct RoundHistoryVO {
    pub index: u32,
    pub round_reward: U128,
    pub reward_time: u64,
    pub avg_reward: u128,
    pub pioneer: Vec<Pioneer>,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct UserInfo {
    pub total_burned: u32,
    pub burned: u32,
    pub pioneer_token_amount: U128,
    pub need_init: bool,
    pub a: Vec<u8>, // record shipsubtype
    pub b: Vec<u8>, // record shipsubtype
    pub c: Vec<u8>, // record shipsubtype
    pub d: Vec<u8>, // record shipsubtype
}
//
#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PioneerToken {
    // shipsubtype -> pioneerTokenAmount
    pioneer_token_a: Vec<u128>, // 4 subtypes
    pioneer_token_b: Vec<u128>, // 8 subtypes
    pioneer_token_c: Vec<u128>, // 16 subtypes
    pioneer_token_d: Vec<u128>, // 32 subtypes
}
//
#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    PioneerHistory,
    PioneerToken,
    UserInfo,
}

#[ext_contract(ext_ft)]
trait FungibleToken {
    fn mint(&mut self, receiver_id: AccountId, amount: U128);
}

#[ext_contract(ext_nft)]
trait NonFungibleToken {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    ship_contract_id: AccountId,
    token_tia: AccountId,


    round_history: Vector<Round>,

    pioneer_history: UnorderedMap<RoundIdAcc, Pioneer>,
    pioneer_token: UnorderedMap<AccountId, PioneerToken>,
    user_info: UnorderedMap<AccountId, UserInfo>,

    total_reward: u128,
    total_claimed_reward: u128,

    pioneer_max: u32,

    balance: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, ship_contract_id: AccountId, token_tia: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id,
            ship_contract_id,
            token_tia,
            round_history: Vector::new(b"r".to_vec()),
            pioneer_history: UnorderedMap::new(StorageKey::PioneerHistory),
            pioneer_token: UnorderedMap::new( StorageKey::PioneerToken),
            user_info: UnorderedMap::new( StorageKey::UserInfo),
        
            total_reward: 0,
            total_claimed_reward: 0,
            pioneer_max: 5, // default: 5
            balance: 0,
        }
    }

    pub fn register_ship(&mut self, token_id: String, token_owner_id: AccountId) {
        require!(env::predecessor_account_id()==self.ship_contract_id, "ERR_NOT_ALLOWED");
        // todo: register business logic
        log!("shippool.register_ship: Register ship {} to owner {}", token_id, token_owner_id);
    }

    // SPL-00-01
    pub fn batch_register_ships(&mut self, token_ids: Vec<String>, token_owner_id: AccountId) {
        require!(env::predecessor_account_id()==self.ship_contract_id, "ERR_NOT_ALLOWED");
        // todo: batch register business logic

        for token_id in token_ids.iter() {
            //
            self.internal_update_pioneer_progress(token_owner_id.clone(), token_id.to_string());
            if true == self.internal_update_epoch_progress(token_owner_id.clone()){
                self.internal_new_epoch();
            }
            //
        }
    }

    #[private]
    pub fn resolve_cross_call(
        &mut self,
    ) -> U128 {
        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(bal) = near_sdk::serde_json::from_slice::<U128>(&value) {
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


    /* ========== CORE FUNCTION ========== */
    // SPL-00-07
    pub fn pioneer_claim(&mut self, round_id: u64)  {
        let predecessor_id = env::predecessor_account_id();

        require!( self.round_history.len() > 0, "ShipPool: no round to claim");
        require!( round_id < self.round_history.len(), "ShipPool: invalid param");

        let round: Round = self.round_history.get(round_id).unwrap().clone();
        require!(round.pioneer.contains(&predecessor_id), "ShipPool: this round no reward can claim");

        let hist_key: RoundIdAcc = format!("{}:{}", round_id, predecessor_id);

        let mut pioneer: Pioneer = self.pioneer_history.get(&hist_key).unwrap().clone();
        require!(!pioneer.claimed, "ShipPool: already claimed");
        pioneer.claimed = true;
        // ssp reward
        let reward: u128 = round.avg_reward * pioneer.pioneer_token_amount as u128 / 1_000_000_000_000;
        //IERC20(ssp).safeTransfer(msg.sender, reward);
        if reward > 0 {
            ext_fungible_token::ft_transfer(
                predecessor_id.clone(),
                U128(reward),
                None,
                self.token_tia.clone(),
                1,
                GAS_FOR_TRANSFER
            );

            self.total_claimed_reward = self.total_claimed_reward + reward;
            self.pioneer_history.insert(&hist_key,&pioneer);
        }
        //emit event
        Event::Claim{caller_id: &env::predecessor_account_id(), round_id, reward: &U128(reward)}.emit();

    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// SPL-00-02
    fn ft_on_transfer(
        &mut self,
        _sender_id: AccountId,
        amount: U128,
        _msg: String,
    ) -> PromiseOrValue<U128> {
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.token_tia, "invalid token contract id");
        self.balance += amount.0;
        //
        PromiseOrValue::Value(U128(0))
    }
}

/* ========== INTERNAL FUNCTION ========== */
impl Contract{
    // SPL-00-03
    pub fn internal_init_user(&mut self, pioneer: AccountId) {
        let mut user: UserInfo = self.user_info.get(&pioneer).unwrap_or(
            UserInfo{
                total_burned: 0,
                burned: 0,
                pioneer_token_amount: U128(0),
                need_init: false,
                a: vec![0;TYPE_A_MAX.into()], // record shipsubtype
                b: vec![0;TYPE_B_MAX.into()], // record shipsubtype
                c: vec![0;TYPE_C_MAX.into()], // record shipsubtype
                d: vec![0;TYPE_D_MAX.into()], // record shipsubtype  
            }
        ).clone();

        let total_burned: u32 = user.total_burned;

        let mut pioneer_token: PioneerToken = self.pioneer_token.get(&pioneer).unwrap().clone();
        for i in 0..user.a.len(){
            pioneer_token.pioneer_token_a[user.a[i] as usize - 1] = 0;
        }
        for j in 0..user.b.len(){
            pioneer_token.pioneer_token_b[user.b[j] as usize - 1] = 0;
        }
        for m in 0..user.c.len(){
            pioneer_token.pioneer_token_c[user.c[m] as usize - 1] = 0;
        }
        for n in 0..user.d.len(){
            pioneer_token.pioneer_token_d[user.d[n] as usize - 1] = 0;
        }
        self.pioneer_token.insert(&pioneer, &pioneer_token);

        //delete userInfo[_pioneer];
        user.total_burned = total_burned;
        user.burned = 0;
        user.pioneer_token_amount = U128(0);
        user.need_init = false;
        user.a = vec![0;TYPE_A_MAX.into()];
        user.b = vec![0;TYPE_B_MAX.into()];
        user.c = vec![0;TYPE_C_MAX.into()];
        user.d = vec![0;TYPE_D_MAX.into()];
        self.user_info.insert(&pioneer, &user);

    }

    pub fn internal_check_pioneer(&mut self, to: AccountId) {
        let idx: u64;
        if self.round_history.len() == 0 {
            // create the first round
            self.round_history.push(&Round{
                pioneer: vec![],
                round_reward: 0,
                total_pioneer_token_amount: 0,
                reward_time: 0,
                avg_reward: 0,
            });
            return;
        }
        if self.round_history.len() == 1{
            idx = self.round_history.len() -1;
        }
        else{
            idx = self.round_history.len() -2;
        }

        let round: Round = self.round_history.get(idx).expect("invalid idx");

        let user: UserInfo = self.user_info.get(&to).expect("invalid key");        
        // if is preRound pioneer check init
        if round.reward_time > 0 && round.pioneer.contains(&to) && user.need_init {
            self.internal_init_user(to);
        }
    }

    // SPL-00-04
    pub fn internal_update_pioneer_progress(&mut self,  to: AccountId, token_id: TokenId ) {
        self.internal_check_pioneer(to.clone());

        let (ship_type, ship_subtype) = self.internal_get_ship_type_subtype_by_token_id(token_id.clone());

        let amount: u128 = self.internal_get_pioneer_token(token_id.clone());
        let mut user: UserInfo = self.user_info.get(&to).unwrap_or(
            UserInfo{
                total_burned: 0,
                burned: 0,
                pioneer_token_amount: U128(0),
                need_init: false,
                a: vec![0;TYPE_A_MAX.into()], // record shipsubtype
                b: vec![0;TYPE_B_MAX.into()], // record shipsubtype
                c: vec![0;TYPE_C_MAX.into()], // record shipsubtype
                d: vec![0;TYPE_D_MAX.into()], // record shipsubtype 
            }
        ).clone();

        require!(ship_type != TYPE_S, "ShipPool: not support type");
        let mut pioneer_token_amount: u128 = 0;
        let mut pioneer_token: PioneerToken = self.pioneer_token.get(&to).unwrap_or(
            PioneerToken{
                pioneer_token_a: vec![0;TYPE_A_MAX.into()], // 4 subtypes
                pioneer_token_b: vec![0;TYPE_B_MAX.into()], // 8 subtypes
                pioneer_token_c: vec![0;TYPE_C_MAX.into()], // 16 subtypes
                pioneer_token_d: vec![0;TYPE_D_MAX.into()], // 32 subtypes
            }
        ).clone();
        if ship_type == TYPE_A {
            pioneer_token_amount = pioneer_token.pioneer_token_a[(ship_subtype-1) as usize];
            if pioneer_token_amount == 0 {
                user.a[(ship_subtype-1) as usize] = ship_subtype;
                user.burned = user.burned + 1;
            }
            pioneer_token.pioneer_token_a[(ship_subtype-1) as usize] = cmp::max(amount, pioneer_token_amount);
        }

        if ship_type == TYPE_B {
            pioneer_token_amount = pioneer_token.pioneer_token_b[(ship_subtype-1) as usize];
            if pioneer_token_amount == 0 {
                user.b[(ship_subtype-1) as usize] = ship_subtype;
                user.burned = user.burned + 1;
            }
            pioneer_token.pioneer_token_b[(ship_subtype-1) as usize] = cmp::max(amount, pioneer_token_amount);
        }

        if ship_type == TYPE_C {
            pioneer_token_amount = pioneer_token.pioneer_token_c[(ship_subtype-1) as usize];
            if pioneer_token_amount == 0 {
                user.c[(ship_subtype-1) as usize] = ship_subtype;
                user.burned = user.burned + 1;
            }
            pioneer_token.pioneer_token_c[(ship_subtype-1) as usize] = cmp::max(amount, pioneer_token_amount);
        }

        if ship_type == TYPE_D {
            pioneer_token_amount = pioneer_token.pioneer_token_d[(ship_subtype-1) as usize];
            if pioneer_token_amount == 0 {
                user.d[(ship_subtype-1) as usize] = ship_subtype;
                user.burned = user.burned + 1;
            }
            pioneer_token.pioneer_token_d[(ship_subtype-1) as usize] = cmp::max(amount, pioneer_token_amount);

        }
        self.pioneer_token.insert(&to.clone(), &pioneer_token); 

        let max: u128 = cmp::max(amount, pioneer_token_amount);
        let min: u128 = cmp::min(amount, pioneer_token_amount);

        // user add up pioneer_token_amount
        if max == pioneer_token_amount{
            user.pioneer_token_amount = user.pioneer_token_amount;
        }
        else{
            user.pioneer_token_amount = U128(user.pioneer_token_amount.0 + max - min);
        }
        self.user_info.insert(&to, &user);
    }

    // SPL-00-05
    pub fn internal_update_epoch_progress(&mut self,  to: AccountId) -> bool {
        let user: UserInfo = self.user_info.get(&to).unwrap().clone();
        if user.burned < ( TYPE_A_MAX + TYPE_B_MAX + TYPE_C_MAX + TYPE_D_MAX ).into() {
            return false;
        }

        let current_round = self.round_history.len()-1;
        let mut round: Round = self.round_history.get(current_round).unwrap().clone();

        // update round pioneer
        let hist_key: RoundIdAcc = format!("{}:{}", current_round, to);

        let mut pioneer: Pioneer = self.pioneer_history.get(&hist_key).unwrap_or(
            Pioneer{
                pioneer: "".to_string(),
                pioneer_token_amount: 0,
                claimed: false,
            }
        ).clone();
        round.pioneer.push(to.clone());

        round.total_pioneer_token_amount = round.total_pioneer_token_amount + user.pioneer_token_amount.0 - pioneer.pioneer_token_amount;
        self.round_history.replace(current_round,&round); // update current round


        pioneer.pioneer = to.to_string();
        pioneer.pioneer_token_amount = user.pioneer_token_amount.0;
        self.pioneer_history.insert(&hist_key, &pioneer);
        return true;
    }

    // SPL-00-06
    pub fn internal_new_epoch(&mut self) {
        let current_round = self.round_history.len()-1;
        let mut round: Round = self.round_history.get(current_round).unwrap().clone();

        let num: u32 = round.pioneer.len() as u32;
        // settle and create new round
        if num < self.pioneer_max {
            return;
        }
        let total: u128 = self.balance;
        let reward: u128 = total+ self.total_claimed_reward - self.total_reward;
        let round_reward: u128 = reward*80/100; 

        round.avg_reward = round_reward*1_000_000_000_000 / round.total_pioneer_token_amount as u128; // TODO: ??????
        round.round_reward = round_reward;
        round.reward_time = nano_to_sec(env::block_timestamp());

        self.round_history.replace(current_round,&round);

        // update distribute reward.
        self.total_reward = self.total_reward + round_reward;

        // create new round
        self.round_history.push(&Round{
            pioneer: vec![],
            round_reward: 0,
            total_pioneer_token_amount: 0,
            reward_time: 0,
            avg_reward: 0,
        });

        // pioneer need init pioneerTokenAmount
        for i in 0..round.pioneer.len() {
            let pioneer: AccountId = round.pioneer[i].clone();
            let mut init_user: UserInfo = self.user_info.get(&pioneer).unwrap().clone();
            init_user.need_init = true;
            self.user_info.insert(&pioneer,&init_user);
        }        
    }

    pub fn internal_get_pioneer_token(& self, token_id: TokenId) -> u128 {
        let items :Vec<&str> = token_id.split(":").collect();    
        let token_id: u128 = items[0].parse::<u128>().unwrap();
        1_000_000_000_000_000_000/(token_id*3) + 1_000_000_000_000_000_000
    }

    pub fn internal_get_ship_type_subtype_by_token_id( &self, token_id: TokenId ) -> (u8,u8) {
        let items :Vec<&str> = token_id.split(":").collect();
        let ship_type: u8 = items[1].parse::<u8>().unwrap();
        let ship_subtype: u8 = items[2].parse::<u8>().unwrap();
        (ship_type, ship_subtype)
    }
}