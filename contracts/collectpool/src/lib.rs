use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet, LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue, Gas, ext_contract, Promise, PromiseResult, log, Balance, serde_json
};
use std::cmp;
use std::convert::{TryFrom, TryInto};

mod view;
mod owner;


// ship type
pub const TYPE_SHIP_S: u8 = 5;
pub const TYPE_SHIP_A: u8 = 4;
pub const TYPE_SHIP_B: u8 = 3;
pub const TYPE_SHIP_C: u8 = 2;
pub const TYPE_SHIP_D: u8 = 1;


pub const TYPE_S_MAX: u8 = 4;
pub const TYPE_A_MAX: u8 = 4;
pub const TYPE_B_MAX: u8 = 8;
pub const TYPE_C_MAX: u8 = 16;
pub const TYPE_D_MAX: u8 = 32;





pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_TRANSFER: Gas = Gas(25* TGAS);
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas(25* TGAS);
pub const GAS_FOR_TRANSFER_ON_CALL: Gas = Gas(45 * TGAS);

pub const YOCTO18: u128 = 1_000_000_000_000_000_000;
pub const CALC_PRECISION: u128 = 1_000_000_000_000;
pub const RATE_DENOMINATOR: u64 = 100;
pub const DEFAULT_DECAY_PERIOD_PER_PHASE: u64 = 3456000;
pub const DEFAULT_DECAY_RATE: u64 = 10;
pub const DEFAULT_REWARD_RATE: u64 = 80;
pub const DEFAULT_FORMAT_RATE: u64 = 100;
pub const DEFAULT_TOTAL_PERIOD: u32 = 12;
pub const DEFAULT_SLOT_FEE2: u32 = 300;
pub const DEFAULT_SLOT_FEE3: u32 = 600;


pub type PriceType = u128;
pub type TimeStampSec = u64;
pub type SlotIndex = u64;

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Slot {
    price: PriceType,
    enable: bool,
    ship_type: u8,
    ship_sub_type: u8,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeSlot {
    token_id: TokenId,
    enable: bool,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SlotList {
    slot_index: u32,
    price: PriceType,
    enable: bool,
    token_id: TokenId,

    capacity: u32,
    ship_type: u8,
    ship_subtype: u8,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    UserRewardPerTokenPaid,
    Rewards,
    SlotInfo,
    CapacityInfo,
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

#[ext_contract(ext_boxmall)]
trait boxmall{
    fn ship_wallet_add( &mut self, to: AccountId, amount: U128 );
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,

    token_tia: AccountId,
    spaceship: AccountId,
    boxmall: AccountId,

    slot:Vector<Slot>,

    /* ========== VARIABLE SETTING ========== */
    start_block: u64,
    end_block: u64,
    last_reward_block: u64,
    total_capacity: u32,

    per_block_reward: u128,

    total_period: u32, // default: 12;
    decay_period: u64, // default: 2592000;
    decay_rate: u64,   // default: 10;
    reward_rate: u64,  // default: 80;
    format_rate: u64,  // default: 100;

    reward_per_token_stored: u128,

    user_reward_per_token_paid: LookupMap<AccountId, u128>, // mapping(address => uint256)
    rewards: LookupMap<AccountId, u128>, // mapping(address => uint256)
    slot_info: LookupMap<AccountId, Vec<StakeSlot>>, // mapping(address => mapping(uint256 => StakeSlot))
    capacity_info: LookupMap<AccountId, u32>,  // mapping(address => uint256) 
    decay_table: Vec<u128>, // mapping(uint256 => uint256)

    is_init_slot: bool,
    balance: Balance,

}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_tia: AccountId, spaceship: AccountId, boxmall: AccountId, start_block: u64, per_block_reward: U128) -> Self {
        require!(!env::state_exists(), "Already initialized");
        let mut this = Contract {
            owner_id,
            token_tia,
            spaceship,
            boxmall,
            slot: Vector::new(b"v".to_vec()),
            start_block,
            end_block: start_block + DEFAULT_TOTAL_PERIOD as u64 * DEFAULT_DECAY_PERIOD_PER_PHASE,
            last_reward_block: 0,
            total_capacity: 0,
            per_block_reward: per_block_reward.0,
            total_period: DEFAULT_TOTAL_PERIOD,
            decay_period: DEFAULT_DECAY_PERIOD_PER_PHASE, 
            decay_rate: DEFAULT_DECAY_RATE, 
            reward_rate: DEFAULT_REWARD_RATE, 
            format_rate: DEFAULT_FORMAT_RATE, 

            reward_per_token_stored: 0,

            user_reward_per_token_paid: LookupMap::new( StorageKey::UserRewardPerTokenPaid),
            rewards: LookupMap::new( StorageKey::Rewards), 
            slot_info:LookupMap::new( StorageKey::SlotInfo), 
            capacity_info: LookupMap::new( StorageKey::CapacityInfo),
            decay_table: vec![0;DEFAULT_TOTAL_PERIOD as usize],

            balance: 0,
            is_init_slot: false,
        };
        this.internal_init_decay_table();
        this
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
    #[payable]
    pub fn withdraw( &mut self, slot_index: u64) {
        assert_one_yocto();
        let predecessor_id = env::predecessor_account_id();
        self.internal_check_start();
        self.internal_update_reward(predecessor_id.clone());

        let mut stake_slot_vec: Vec<StakeSlot> = self.slot_info.get(&predecessor_id).expect("No_StakeSlot");

        let mut withdraw_slot: StakeSlot = stake_slot_vec[slot_index as usize].clone();
        require!(withdraw_slot.token_id != "".to_string(), "No ship can withdraw");

        ext_nft::nft_transfer(
            predecessor_id.clone(), 
            withdraw_slot.token_id.clone(),
            None,
            None,
            self.spaceship.clone(),
            1,
            GAS_FOR_NFT_TRANSFER
        );


        let mut capacity = self.capacity_info.get(&predecessor_id.clone()).unwrap_or(0);
        // check extra capacity and remove
        let extra: u32 = self.get_extra_capacity(predecessor_id.clone(), slot_index);
        capacity -= extra;
        self.total_capacity -= extra; 

        let ship_capacity = self.internal_get_ship_capacity_by_token_id(withdraw_slot.token_id.clone());
        capacity -= ship_capacity as u32;
        self.capacity_info.insert(&predecessor_id.clone(), &capacity);
        self.total_capacity -= ship_capacity as u32;       
        withdraw_slot.token_id = "".to_string();
        stake_slot_vec[slot_index as usize] = withdraw_slot;

        self.slot_info.insert(&predecessor_id,&stake_slot_vec);
    }

    #[payable]
    pub fn claim( &mut self ) {
        assert_one_yocto();
        let predecessor_id = env::predecessor_account_id();
        self.internal_check_start();
        self.internal_update_reward(predecessor_id.clone());

        let reward: u128 = self.earned(predecessor_id.clone());
        require!(reward > 0, "no reward can claim");

        // most reward into user wallet
        let actual: u128 = reward * self.reward_rate as u128 / self.format_rate as u128;
        ext_fungible_token::ft_transfer(
            predecessor_id.clone(), 
            U128(actual), 
            None, 
            self.token_tia.clone(), 
            1, 
            GAS_FOR_TRANSFER
        );

        // some reward into shipWallet
        // trigger boxmall::ft_on_transfer to add reward to shipwallet
        let mut msg: String = String::from(r#"{"contract_id": "collectpool","user_id": ""#);
        msg += &predecessor_id.to_string();
        msg += &r#""}"#.to_string();

        ext_fungible_token::ft_transfer_call(
            self.boxmall.clone(), 
            U128(reward - actual), 
            None, 
            msg,
            self.token_tia.clone(), 
            1, 
            GAS_FOR_TRANSFER_ON_CALL
        );

        self.rewards.insert(&predecessor_id.clone(),&0);
    }

    #[payable]
    pub fn init_slot(&mut self) {
        assert_one_yocto();
        self.assert_owner();

        require!(!self.is_init_slot, "already init");

        for d in 1..(TYPE_D_MAX+1) {
            self.slot.push(&Slot{price : 0, enable : true, ship_type : TYPE_SHIP_D, ship_sub_type : d});
        }

        for c in 1..(TYPE_C_MAX+1) {
            self.slot.push(&Slot{price : 0, enable : true, ship_type : TYPE_SHIP_C, ship_sub_type : c});
        }

        for b in 1..(TYPE_B_MAX+1) {
            self.slot.push(&Slot{price : 0, enable : true, ship_type : TYPE_SHIP_B, ship_sub_type : b});
        }

        for a in 1..(TYPE_A_MAX+1) {
            self.slot.push(&Slot{price : 0, enable : true, ship_type : TYPE_SHIP_A, ship_sub_type : a});
        }

        for s in 1..(TYPE_S_MAX+1) {
            self.slot.push(&Slot{price : 0, enable : true, ship_type : TYPE_SHIP_S, ship_sub_type : s});
        }
        self.is_init_slot = true;
    }
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    /// Callback on receiving NFT tokens by this contract.
    /// BML-00-01
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>
    {
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.spaceship, "Invalid contract Id");

        let slot_index: SlotIndex = msg.parse::<SlotIndex>().expect("msg must contain all digits");

        self.internal_stake(sender_id, slot_index, token_id.clone());

        PromiseOrValue::Value(false)
    }
}

impl Contract{
    pub fn internal_stake( &mut self, sender_id: AccountId, slot_index: u64, token_id: TokenId) {
        self.internal_check_start();
        self.internal_check_slot(sender_id.clone(), slot_index, token_id.clone());
        self.internal_update_reward(sender_id.clone());

        let mut stake_slot_vec: Vec<StakeSlot> = self.slot_info.get(&sender_id).unwrap_or(
            vec![StakeSlot{
                token_id:"".to_string(),
                enable: false};
                ( TYPE_A_MAX + TYPE_B_MAX + TYPE_C_MAX + TYPE_D_MAX + TYPE_S_MAX ) as usize
                ]
        );

        let mut stake_slot: StakeSlot = stake_slot_vec[slot_index as usize].clone();
        if stake_slot.token_id != "".to_string() {
            self.withdraw(slot_index);
        }

        stake_slot.token_id = token_id.clone();
        stake_slot_vec[slot_index as usize] = stake_slot.clone();
        self.slot_info.insert(&sender_id,& stake_slot_vec);

        let mut capacity = self.capacity_info.get(&sender_id).unwrap_or(0);
        let ship_capacity = self.internal_get_ship_capacity_by_token_id(token_id.clone());
        capacity += ship_capacity as u32;
        self.total_capacity += ship_capacity as u32;

        // check extra capacity and add
        let extra: u32 = self.get_extra_capacity(sender_id.clone(), slot_index);
        capacity += extra;
        self.capacity_info.insert(&sender_id, &capacity);
        self.total_capacity += extra;
    }


    pub fn internal_check_slot( &self, from: AccountId, slot_index: u64, token_id: TokenId) {
        let slot_temp: Slot = self.slot.get(slot_index).expect("Invalid slot_index");
        let ship_type = self.internal_get_ship_type_by_token_id(token_id.clone());
        let ship_sub_type = self.internal_get_ship_subtype_by_token_id(token_id.clone());

        require!(slot_temp.ship_type == ship_type && slot_temp.ship_sub_type == ship_sub_type, "Slot not matched");
    }

    pub fn internal_update_reward( &mut self, from: AccountId ) {
        self.reward_per_token_stored = self.reward_per_token();
        self.last_reward_block = self.get_last_block_applicable();
        if from != "00".parse().unwrap() {
            self.rewards.insert(&from.clone(), &self.earned(from.clone()));
            self.user_reward_per_token_paid.insert(&from.clone(), &self.reward_per_token_stored);
        }
    }

    pub fn internal_check_start( &self ) {
        require!(env::block_height() >= self.start_block, "not start");
    }
}

/* ========== INTERNAL FUNCTION ========== */
impl Contract{
    pub fn internal_init_decay_table(&mut self) {
        let mut decay: u128 = 0;

        for i in 0..self.total_period{
            if i == 0{
                decay = self.per_block_reward;
            }
            else{
                decay = self.decay_table[(i-1) as usize]*(self.format_rate as u128 - self.decay_rate as u128)/self.format_rate as u128;
            }
            self.decay_table[i as usize] = decay;
        }
    }


    pub fn internal_get_ship_type_by_token_id( &self, token_id: TokenId ) -> u8 {
        let items :Vec<&str> = token_id.split(":").collect();    
        let ship_type: u8 = items[1].parse::<u8>().unwrap();
        ship_type
    }

    pub fn internal_get_ship_subtype_by_token_id( &self, token_id: TokenId ) -> u8 {
        let items :Vec<&str> = token_id.split(":").collect();    
        let ship_subtype: u8 = items[2].parse::<u8>().unwrap();
        ship_subtype
    }

    pub fn internal_get_ship_capacity_by_token_id( &self, token_id: TokenId ) -> u8 {
        let items :Vec<&str> = token_id.split(":").collect();    
        let capacity: u8 = items[3].parse::<u8>().unwrap();
        capacity
    }

    pub fn get_last_block_applicable( &self ) -> u64 {
        return cmp::min(env::block_height(), self.end_block);
    }

    pub fn reward_per_token( &self ) -> u128 {
        if self.total_capacity == 0 {
            return self.reward_per_token_stored;
        }
        return self.reward_per_token_stored + self.calc_block_reward(self.last_reward_block)*CALC_PRECISION/self.total_capacity as u128;
    }

    pub fn earned( &self, from: AccountId) -> u128 {
        let capacity: u64 = self.capacity_info.get(&from).unwrap_or(0) as u64;
        let reward_per_token: u128 = self.reward_per_token();
        let user_reward_per_token_paid: u128 = self.user_reward_per_token_paid.get(&from).unwrap_or(0);
        let reward: u128 = self.rewards.get(&from).unwrap_or(0);

        capacity as u128*(reward_per_token - user_reward_per_token_paid)/CALC_PRECISION + reward
    }

   // return period of current block
   pub fn phase( &self, block: u64 ) -> u64 {
        if self.decay_period == 0 {
            return 0;
        }
        if block > self.start_block {
            return (block - self.start_block - 1)/self.decay_period;
        }
        return 0;
    }

    pub fn calc_block_reward( &self, last_reward_block: u64) -> u128 {
        let mut block_reward: u128 = 0;
        let mut n = self.phase(last_reward_block);
        let m = self.phase(self.get_last_block_applicable());
        let mut last_reward_block = last_reward_block.clone();

        while n < m {
            n+=1;
            let r: u64 = n * self.decay_period+ self.start_block;
            block_reward = block_reward + (r as u128 - last_reward_block as u128)*self.get_decay_block_reward(r);
            last_reward_block = r;
        }
        block_reward = block_reward + (self.get_last_block_applicable() as u128 - last_reward_block as u128)*(self.get_decay_block_reward(self.get_last_block_applicable()));
        return block_reward;
    }

    pub fn get_decay_block_reward( &self,  block_number: u64) -> u128 {
        let period:u64 = self.phase(block_number);
        return self.decay_table[period as usize];
    }
}