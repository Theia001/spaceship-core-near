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
    PromiseOrValue, Gas, ext_contract, Promise, PromiseResult, log, Balance
};
use std::cmp;

mod view;
mod owner;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_TRANSFER: Gas = Gas(25 * TGAS);
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas(25 * TGAS);
pub const GAS_FOR_TRANSFER_ON_CALL: Gas = Gas(45 * TGAS);

pub const YOCTO18: u128 = 1_000_000_000_000_000_000;
pub const CALC_PRECISION: u128 = 1_000_000_000_000;
pub const RATE_DENOMINATOR: u64 = 100;
pub const DEFAULT_DECAY_PERIOD_PER_PHASE: u64 = 3456000;
pub const DEFAULT_DECAY_RATE: u64 = 10;
pub const DEFAULT_REWARD_RATE: u64 = 80;
pub const DEFAULT_FORMAT_RATE: u64 = 100;
pub const DEFAULT_TOTAL_PERIOD: u32 = 12;
pub const DEFAULT_SLOT_FEE1: u32 = 300;
pub const DEFAULT_SLOT_FEE2: u32 = 600;

pub type PriceType = u128;
pub type TimeStampSec = u64;
pub type SlotIndex = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Slot {
    price: PriceType,
    enable: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StakeSlot {
    token_id: TokenId,
    enable: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
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
    BalanceInfo,
    BalanceBuffer,
    CapacityBuffer,
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
trait boxmall {
    fn ship_wallet_add(&mut self, to: AccountId, amount: U128);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,

    burned_addr: AccountId, //default = AccountId("00");
    token_tia: AccountId,
    usn: AccountId,
    spaceship: AccountId,
    boxmall: AccountId,

    slot: Vector<Slot>,

    /* ========== VARIABLE SETTING ========== */
    start_block: u64,
    end_block: u64,
    last_reward_block: u64,
    total_capacity: u32,
    total_supply: u128,

    per_block_reward: u128,

    total_period: u32,  // default: 12
    decay_period: u64,  // default: 3456000
    decay_rate: u64,    // default: 10
    reward_rate: u64,   // default: 80
    format_rate: u64,   // default: 100

    slot_fee2: u128,
    slot_fee3: u128,

    reward_per_token_stored: u128,

    user_reward_per_token_paid: LookupMap<AccountId, u128>, // mapping(address => uint256)
    rewards: LookupMap<AccountId, u128>,                    // mapping(address => uint256)
    slot_info: LookupMap<AccountId, Vec<StakeSlot>>, // mapping(address => mapping(uint256 => StakeSlot))

    balance_info: LookupMap<AccountId, u128>, // mapping(address => uint256)
    balance_buffer: LookupMap<AccountId, u128>, // mapping(address => uint256)
    capacity_buffer: LookupMap<AccountId, u32>, // mapping(address => uint256)
    capacity_info: LookupMap<AccountId, u32>, // mapping(address => uint256)
    decay_table: Vec<u128>,                   // mapping(uint256 => uint256)

    balance: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        token_tia: AccountId,
        usn: AccountId,
        spaceship: AccountId,
        boxmall: AccountId,
        start_block: u64,
        per_block_reward: U128,
        slot_fee2: U128,
        slot_fee3: U128,
    ) -> Self {
        require!(!env::state_exists(), "Already initialized");
        let mut this = Contract {
            owner_id,
            burned_addr: "00".parse().unwrap(),
            token_tia,
            usn,
            spaceship,
            boxmall,
            slot: Vector::new(b"v".to_vec()),
            start_block,
            end_block: start_block + DEFAULT_TOTAL_PERIOD as u64 * DEFAULT_DECAY_PERIOD_PER_PHASE,
            last_reward_block: 0,
            total_supply: 0,
            total_capacity: 0,
            per_block_reward: per_block_reward.0,
            total_period: DEFAULT_TOTAL_PERIOD,
            decay_period: DEFAULT_DECAY_PERIOD_PER_PHASE,
            decay_rate: DEFAULT_DECAY_RATE,
            reward_rate: DEFAULT_REWARD_RATE,
            format_rate: DEFAULT_FORMAT_RATE,

            reward_per_token_stored: 0,

            slot_fee2: slot_fee2.0,
            slot_fee3: slot_fee3.0,

            user_reward_per_token_paid: LookupMap::new(StorageKey::UserRewardPerTokenPaid),
            rewards: LookupMap::new(StorageKey::Rewards),
            slot_info: LookupMap::new(StorageKey::SlotInfo),
            balance_info: LookupMap::new(StorageKey::BalanceInfo),
            balance_buffer: LookupMap::new(StorageKey::BalanceBuffer),
            capacity_buffer: LookupMap::new(StorageKey::CapacityBuffer),
            capacity_info: LookupMap::new(StorageKey::CapacityInfo),
            decay_table: vec![0; DEFAULT_TOTAL_PERIOD as usize],

            balance: 0,
        };
        this.internal_init_decay_table();
        this.internal_init_slot();
        this
    }

    /* ========== CORE FUNCTION ========== */
    pub fn withdraw_amount( &mut self, amount: u128 ) {
        let predecessor_id = env::predecessor_account_id();
        self.internal_check_start();
        self.internal_update_reward(predecessor_id.clone());

        ext_fungible_token::ft_transfer(
            predecessor_id.clone(),
            U128(amount),
            None,
            self.usn.clone(),
            0,
            GAS_FOR_TRANSFER,
        );

        let mut capacity_info = self.capacity_info.get(&predecessor_id.clone()).unwrap_or(0);
        let mut capacity_buffer = self
            .capacity_buffer
            .get(&predecessor_id.clone())
            .unwrap_or(0);
        let mut balance_info = self.balance_info.get(&predecessor_id.clone()).unwrap_or(0);
        let mut balance_buffer = self
            .balance_buffer
            .get(&predecessor_id.clone())
            .unwrap_or(0);

        if capacity_info > 0 {
            self.total_supply -= amount * (capacity_info as u128 + 100) / 100;

            balance_info -= amount;
            self.balance_info
                .insert(&predecessor_id.clone(), &balance_info);
        } else {
            balance_buffer -= amount;
            self.balance_buffer
                .insert(&predecessor_id.clone(), &balance_buffer);
        }

        // balance is 0 capacity back to buffer
        if balance_info == 0 {
            self.total_capacity -= capacity_info;

            capacity_buffer += capacity_info;
            self.capacity_buffer
                .insert(&predecessor_id.clone(), &capacity_buffer);

            capacity_info = 0;
            self.capacity_info
                .insert(&predecessor_id.clone(), &capacity_info);
        }
    }

    #[payable]
    pub fn withdraw(&mut self, slot_index: u64) {
        assert_one_yocto();
        let predecessor_id = env::predecessor_account_id();
        self.internal_check_start();
        self.internal_update_reward(predecessor_id.clone());

        let mut stake_slot_vec: Vec<StakeSlot> =
            self.slot_info.get(&predecessor_id).expect("No_StakeSlot");

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

        let mut capacity_info = self.capacity_info.get(&predecessor_id.clone()).unwrap_or(0);
        let mut capacity_buffer = self
            .capacity_buffer
            .get(&predecessor_id.clone())
            .unwrap_or(0);
        let balance_info = self.balance_info.get(&predecessor_id.clone()).unwrap_or(0);
        let ship_capacity =
            self.internal_get_ship_capacity_by_token_id(withdraw_slot.token_id.clone());

        if balance_info > 0 {
            self.total_supply -= balance_info * (capacity_info as u128 + 100) / 100;

            capacity_info -= ship_capacity as u32;
            self.capacity_info
                .insert(&predecessor_id.clone(), &capacity_info);

            self.total_capacity -= ship_capacity as u32;
        } else {
            capacity_buffer -= ship_capacity as u32;
            self.capacity_buffer
                .insert(&predecessor_id.clone(), &capacity_buffer);
        }

        withdraw_slot.token_id = "".to_string();
        stake_slot_vec[slot_index as usize] = withdraw_slot;
        self.slot_info.insert(&predecessor_id, &stake_slot_vec);

        self.total_supply += balance_info * (capacity_info as u128 + 100) / 100;
    }

    #[payable]
    pub fn claim(&mut self) {
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
        let mut msg: String = String::from(r#"{"contract_id": "corepool","user_id": ""#);
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

        self.rewards.insert(&predecessor_id.clone(), &0);
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.usn || predecessor_id == self.token_tia, "Invalid contract Id");

        if predecessor_id == self.usn {
            self.internal_stake_amount(sender_id.clone(), amount.0);
        }
        if predecessor_id == self.token_tia {
            let slot_index: u64 = msg.parse::<u64>().expect("msg must contain all digits");
            self.internal_buy_slot(sender_id.clone(), amount.0, slot_index);
        }

        PromiseOrValue::Value(U128(0))
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
    ) -> PromiseOrValue<bool> {
        let predecessor_id = env::predecessor_account_id();
        require!(predecessor_id == self.spaceship, "Invalid contract Id");

        let slot_index: SlotIndex = msg
            .parse::<SlotIndex>()
            .expect("msg must contain all digits");

        self.internal_stake(sender_id, slot_index, token_id.clone());

        PromiseOrValue::Value(false)
    }
}

impl Contract{
    pub fn internal_buy_slot( &mut self, sender_id: AccountId, amount: u128, slot_index: u64)  {
        let slot_temp: Slot = self.slot.get(slot_index).expect("Invalid slot_index");
        require!(!slot_temp.enable, "Slot is enable");
        require!(slot_temp.price == amount, "price is invalid");

        let mut stake_slot_vec: Vec<StakeSlot> =
            self.slot_info.get(&sender_id).unwrap_or(vec![
                StakeSlot {
                    token_id: "".to_string(),
                    enable: false
                };
                4
            ]);

        let mut stake: StakeSlot = stake_slot_vec[slot_index as usize].clone();

        ext_fungible_token::ft_transfer(
            self.burned_addr.clone(), 
            U128(slot_temp.price), 
            None, 
            self.token_tia.clone(), 
            1, 
            GAS_FOR_TRANSFER
        );
 
        stake.enable = true;
        stake_slot_vec[slot_index as usize] = stake;

        self.slot_info.insert(&sender_id, &stake_slot_vec);

    }

    pub fn internal_stake( &mut self, sender_id: AccountId, slot_index: u64, token_id: TokenId) {
        self.internal_check_start();
        self.internal_check_slot(sender_id.clone(), slot_index);
        self.internal_update_reward(sender_id.clone());

        let mut stake_slot_vec: Vec<StakeSlot> =
            self.slot_info.get(&sender_id).expect("No_StakeSlot");

        let mut stake_slot: StakeSlot = stake_slot_vec[slot_index as usize].clone();
        if stake_slot.token_id != "".to_string() {
            self.withdraw(slot_index);
        }

        stake_slot.token_id = token_id.clone();
        stake_slot_vec[slot_index as usize] = stake_slot.clone();
        self.slot_info.insert(&sender_id.clone(), &stake_slot_vec);

        let mut capacity_info = self.capacity_info.get(&sender_id).unwrap_or(0);
        let mut capacity_buffer = self.capacity_buffer.get(&sender_id).unwrap_or(0);
        let mut balance_info = self.balance_info.get(&sender_id).unwrap_or(0);
        let mut balance_buffer = self.balance_buffer.get(&sender_id).unwrap_or(0);
        let ship_capacity = self.internal_get_ship_capacity_by_token_id(token_id.clone());

        if balance_info == 0 && balance_buffer == 0 {
            capacity_buffer += ship_capacity as u32;
            self.capacity_buffer.insert(&sender_id, &capacity_buffer);
        } else {
            self.total_supply =
                self.total_supply - balance_info * (capacity_info as u128 + 100) / 100;

            capacity_info += ship_capacity as u32 + capacity_buffer;
            self.capacity_info.insert(&sender_id, &capacity_info);

            self.total_capacity += ship_capacity as u32 + capacity_buffer;

            capacity_buffer = 0;
            self.capacity_buffer.insert(&sender_id, &capacity_buffer);

            // reset balance
            balance_info += balance_buffer;
            self.balance_info.insert(&sender_id, &balance_info);

            balance_buffer = 0;
            self.balance_buffer.insert(&sender_id, &balance_buffer);

            self.total_supply += balance_info * (capacity_info as u128 + 100) / 100;
        }
    }

    pub fn internal_stake_amount( &mut self, sender_id: AccountId, amount: u128 ) {
        self.internal_check_start();
        self.internal_update_reward(sender_id.clone());

        let mut capacity_info = self.capacity_info.get(&sender_id).unwrap_or(0);
        let mut capacity_buffer = self.capacity_buffer.get(&sender_id).unwrap_or(0);
        let mut balance_info = self.balance_info.get(&sender_id).unwrap_or(0);
        let mut balance_buffer = self.balance_buffer.get(&sender_id).unwrap_or(0);


        if capacity_info == 0 && capacity_buffer == 0 {
            balance_buffer += amount;
            self.balance_buffer.insert(&sender_id, &balance_buffer);
        }else{
            self.total_supply = self.total_supply - balance_info*(capacity_info as u128 + 100)/100;

            balance_info += amount + balance_buffer;
            self.balance_info.insert(&sender_id, &balance_info);

            balance_buffer = 0;
            self.balance_buffer.insert(&sender_id, &balance_buffer);

            self.total_capacity += capacity_buffer;
            // reset capacity
            capacity_info += capacity_buffer;
            self.capacity_info.insert(&sender_id, &capacity_info);

            capacity_buffer = 0;
            self.capacity_buffer.insert(&sender_id, &capacity_buffer);

            self.total_supply += balance_info * (capacity_info as u128 + 100) / 100;
        }
    }

    pub fn internal_check_slot(&self, from: AccountId, slot_index: u64) {
        let slot_temp: Slot = self.slot.get(slot_index).expect("Invalid slot_index");
        let stake_slot_vec: Vec<StakeSlot> = self.slot_info.get(&from).expect("No_StakeSlot");

        let stake: StakeSlot = stake_slot_vec[slot_index as usize].clone();
        require!(stake.enable || slot_temp.enable, "Slot is not enable");
    }

    pub fn internal_update_reward(&mut self, from: AccountId) {
        self.reward_per_token_stored = self.reward_per_token();
        self.last_reward_block = self.get_last_block_applicable();
        if from != "00".parse().unwrap() {
            self.rewards
                .insert(&from.clone(), &self.earned(from.clone()));
            self.user_reward_per_token_paid
                .insert(&from.clone(), &self.reward_per_token_stored);
        }
    }

    pub fn internal_check_start(&self) {
        require!(env::block_height() >= self.start_block, "not start");
    }
}

/* ========== INTERNAL FUNCTION ========== */
impl Contract {
    pub fn internal_init_decay_table(&mut self) {
        let mut decay: u128 = 0;

        for i in 0..self.total_period {
            if i == 0 {
                decay = self.per_block_reward;
            } else {
                decay = self.decay_table[(i - 1) as usize]
                    * (self.format_rate as u128 - self.decay_rate as u128)
                    / self.format_rate as u128;
            }
            self.decay_table[i as usize] = decay;
        }
    }

    pub fn internal_init_slot(&mut self) {
        self.slot.push(&Slot {
            price: 0,
            enable: true,
        });
        self.slot.push(&Slot {
            price: 0,
            enable: true,
        });
        self.slot.push(&Slot {
            price: self.slot_fee2,
            enable: false,
        });
        self.slot.push(&Slot {
            price: self.slot_fee3,
            enable: false,
        });
    }

    pub fn internal_get_ship_type_by_token_id(&self, token_id: TokenId) -> u8 {
        let items: Vec<&str> = token_id.split(":").collect();
        let ship_type: u8 = items[1].parse::<u8>().unwrap();
        ship_type
    }

    pub fn internal_get_ship_subtype_by_token_id(&self, token_id: TokenId) -> u8 {
        let items: Vec<&str> = token_id.split(":").collect();
        let ship_subtype: u8 = items[2].parse::<u8>().unwrap();
        ship_subtype
    }

    pub fn internal_get_ship_capacity_by_token_id(&self, token_id: TokenId) -> u8 {
        let items: Vec<&str> = token_id.split(":").collect();
        let capacity: u8 = items[3].parse::<u8>().unwrap();
        capacity
    }

    pub fn get_last_block_applicable(&self) -> u64 {
        return cmp::min(env::block_height(), self.end_block);
    }

    pub fn reward_per_token(&self) -> u128 {
        if self.total_supply == 0 || self.total_capacity == 0 {
            return self.reward_per_token_stored;
        }
        return self.reward_per_token_stored
            + self.calc_block_reward(self.last_reward_block) * CALC_PRECISION / self.total_supply;
    }

    pub fn earned( &self, from: AccountId) -> u128 {
        let capacity: u64 = self.capacity_info.get(&from).unwrap_or(0) as u64;
        let balance: u128 = self.balance_info.get(&from).unwrap_or(0) as u128;
        let reward_per_token: u128 = self.reward_per_token();
        let user_reward_per_token_paid: u128 = self.user_reward_per_token_paid.get(&from).unwrap_or(0);
        let reward: u128 = self.rewards.get(&from).unwrap_or(0);

        ((( capacity as u128+100 ) * balance)/100 ) * ( reward_per_token - user_reward_per_token_paid)/CALC_PRECISION + reward
    }

    // return period of current block
    pub fn phase(&self, block: u64) -> u64 {
        if self.decay_period == 0 {
            return 0;
        }
        if block > self.start_block {
            return (block - self.start_block - 1) / self.decay_period;
        }
        return 0;
    }

    pub fn calc_block_reward(&self, last_reward_block: u64) -> u128 {
        let mut block_reward: u128 = 0;
        let mut n = self.phase(last_reward_block);
        let m = self.phase(self.get_last_block_applicable());
        let mut last_reward_block = last_reward_block.clone();

        while n < m {
            n += 1;
            let r: u64 = n * self.decay_period + self.start_block;
            block_reward = block_reward
                + (r as u128 - last_reward_block as u128) * self.get_decay_block_reward(r);
            last_reward_block = r;
        }
        block_reward = block_reward
            + (self.get_last_block_applicable() as u128 - last_reward_block as u128)
                * (self.get_decay_block_reward(self.get_last_block_applicable()));
        return block_reward;
    }

    pub fn get_decay_block_reward(&self, block_number: u64) -> u128 {
        let period: u64 = self.phase(block_number);
        return self.decay_table[period as usize];
    }
}