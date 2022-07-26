use std::borrow::Borrow;
use near_sdk::{near_bindgen, BorshStorageKey, PanicOnDefault, serde_json, env, AccountId, PromiseOrValue, Gas, ext_contract, assert_one_yocto, require, log};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::json_types::U128;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::non_fungible_token::{TokenId};
use std::collections::HashMap;
use rand::{Rng,SeedableRng};
use rand::rngs::StdRng;

mod view;
mod owner;
mod utils;
mod events;
pub use crate::utils::*;
pub use crate::events::*;

pub type  TimeStampSec = u64;
pub const YOCTO18: u128 = 1_000_000_000_000_000_000;
pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas(30 * TGAS);
pub const GAS_FOR_BATCH_TRANSFER: Gas = Gas(30 * TGAS);
pub const GAS_FOR_BATCH_TRANSFER_CALL: Gas = Gas(35 * TGAS);
pub const GAS_FOR_SPACESHIP_UPGRADE: Gas = Gas(100 * TGAS);

pub const RATE_DENOMINATOR: u8 = 100;

// ship type
pub const TYPE_S: u8 = 5;
pub const TYPE_A: u8 = 4;
pub const TYPE_B: u8 = 3;
pub const TYPE_C: u8 = 2;
pub const TYPE_D: u8 = 1;


// Third party contracts section
#[ext_contract(ext_spaceship)]
pub trait Spaceship {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );

    fn upgrade_spaceship(
        &mut self, 
        owner_id: AccountId, 
        token_id_1: TokenId, 
        token_id_2: TokenId, 
        target_sub_type: u8, 
        eng_amount: U128
    );
}

#[ext_contract(ext_tokentia)]
pub trait TokenTia{
    fn batch_transfer(
        &mut self,
        receiver_ids: Vec<String>,
        amounts: Vec<U128>,
        memo: Option<String>,
    );
}

pub enum OrderStatus {
    OrderSell = 0,
    OrderBuy = 1,
    OrderCancel = 2,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TransferCallInfo {
    BuyInfo{ order_id: u64 },
    UpgradeInfo{ token_id_1: TokenId, token_id_2: TokenId, target_sub_type: u8},
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    order_id: u64,
    seller: AccountId,
    buyer: AccountId,
    token_id: TokenId,
    status: u8, // 0 selling; 1 buy success; 2 cancel
    #[serde(with = "u128_dec_format")]
    amount: u128,
    create_time: TimeStampSec,
    update_time: TimeStampSec,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    OrderMapKey,
    SellOrderMapKey,
    BuyOrderMapKey,
    NoTargetMintFeeKey,
    TargetMintFeeKey,
    TargetSubTypeKey
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    //
    next_order_id: u64,
    ship_reward_rate: u8,
    risker_reward_rate: u8,
    bank_reward_rate: u8,
    luck_reward_rate: u8,
    rank_reward_rate: u8,
    fee_rate: u8,
    shift: usize,

    spaceship: AccountId,
    token_tia: AccountId,
    ship_pool: AccountId,
    bank: AccountId,
    risker_pool: AccountId,
    rank_pool: AccountId,
    luck_pool: AccountId,

    no_target_mint_fee: UnorderedMap<u8, Vec<u128>>, // arr[0] is token_tia fee arr[1] is eng fee
    target_mint_fee: UnorderedMap<u8, Vec<u128>>, // arr[0] is token_tia fee arr[1] is eng fee
    target_sub_type: UnorderedMap<u8, Vec<u8>>, // array is [min, max] subType range
    order_map: UnorderedMap<u64, Order>,
    sell_order_map: UnorderedMap<AccountId, Vec<u64>>,
    buy_order_map: UnorderedMap<AccountId, Vec<u64>>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, spaceship: AccountId, token_tia: AccountId, ship_pool: AccountId, bank: AccountId,
               risker_pool: AccountId, rank_pool: AccountId, luck_pool: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut temp_no_target_mint_fee: UnorderedMap<u8, Vec<u128>> = UnorderedMap::new(StorageKey::NoTargetMintFeeKey);
        temp_no_target_mint_fee.insert(&TYPE_B, &vec![400 * YOCTO18, 4* YOCTO18].into());
        temp_no_target_mint_fee.insert(&TYPE_C, &vec![200 * YOCTO18, 2 * YOCTO18].into());
        temp_no_target_mint_fee.insert(&TYPE_D, &vec![100 * YOCTO18, 1 * YOCTO18].into());

        let mut temp_target_mint_fee: UnorderedMap<u8, Vec<u128>> = UnorderedMap::new(StorageKey::TargetMintFeeKey);
        temp_target_mint_fee.insert(&TYPE_B, &vec![600 * YOCTO18, 8 * YOCTO18].into());
        temp_target_mint_fee.insert(&TYPE_C, &vec![300 * YOCTO18, 4 * YOCTO18].into());
        temp_target_mint_fee.insert(&TYPE_D, &vec![150 * YOCTO18, 2 * YOCTO18].into());

        let mut temp_target_sub_type: UnorderedMap<u8, Vec<u8>> = UnorderedMap::new(StorageKey::TargetSubTypeKey);
        temp_target_sub_type.insert(&TYPE_B, &vec![1, 4]);
        temp_target_sub_type.insert(&TYPE_C, &vec![1, 8]);
        temp_target_sub_type.insert(&TYPE_D, &vec![1, 16]);

        Contract {
            owner_id,

            //
            next_order_id: 0,
            fee_rate: 0,
            ship_reward_rate: 30,
            risker_reward_rate: 5,
            bank_reward_rate: 5,
            luck_reward_rate: 5,
            rank_reward_rate: 5,
            shift: 0,

            // other contract(account)
            spaceship,
            token_tia,
            ship_pool,
            bank,
            risker_pool,
            rank_pool,
            luck_pool,
            //

            no_target_mint_fee: temp_no_target_mint_fee,
            target_mint_fee: temp_target_mint_fee,
            target_sub_type: temp_target_sub_type,
            order_map: UnorderedMap::new(StorageKey::OrderMapKey),
            sell_order_map: UnorderedMap::new(StorageKey::SellOrderMapKey),
            buy_order_map: UnorderedMap::new(StorageKey::BuyOrderMapKey),
        }
    }

    // 取消出售
    pub fn user_cancel_sell_spaceship(&mut self, order_id: u64) {
        let predecessor_id = env::predecessor_account_id();
        let order = self.order_map.get(order_id.borrow());

        if let Some(mut order) = order {
            require!(order.seller == predecessor_id, "Market: invalid user");
            require!(order.status == OrderStatus::OrderSell as u8, "Market: invalid order status");

            order.update_time = nano_to_sec(env::block_timestamp());
            order.status = OrderStatus::OrderCancel as u8;
            self.order_map.insert(order_id.borrow(), order.borrow());

            ext_spaceship::nft_transfer(
                order.seller.clone(),
                order.token_id,
                None,
                None,
                self.spaceship.clone(),
                1,
                GAS_FOR_NFT_TRANSFER
            );

            // emit CancelEvent
            Event::CancelEvent{
                order_id, 
                seller_id: &order.seller, 
                order_status: OrderStatus::OrderCancel as u8, 
                update_time: order.update_time
            }.emit();
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        // check who call this contract
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.spaceship , "Invalid nft contract Id");
        self.internal_sell_spaceship(previous_owner_id, token_id, msg)
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // check who call this contract
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.token_tia , "Invalid ft contract Id");

        let info: TransferCallInfo = serde_json::from_str::<TransferCallInfo>(&msg).expect("invalid msg");

        match info {
            TransferCallInfo::BuyInfo{order_id} => {
                self.internal_buy_spaceship(sender_id, order_id, amount);
            },
            TransferCallInfo::UpgradeInfo{ token_id_1, token_id_2, target_sub_type} => {
                self.internal_upgrade_spaceship(sender_id, token_id_1, token_id_2, target_sub_type, amount);
            },
        }

        PromiseOrValue::Value(U128(0))
    }
}


impl Contract {

    /// 出售飞船
    pub fn internal_sell_spaceship(&mut self, previous_owner_id: AccountId, token_id: TokenId, msg: String) -> PromiseOrValue<bool> {
        let amount = msg.parse::<u128>().expect("msg must contain all digits");
        self.next_order_id += 1;
        let order_id = self.next_order_id;
        let timestamp = nano_to_sec(env::block_timestamp());
        let new_order: Order = Order {
            order_id,
            seller: previous_owner_id.clone(),
            buyer: "00".parse().unwrap(), //env::current_id()
            token_id: token_id.clone(),
            status: OrderStatus::OrderSell as u8,
            amount,
            create_time: timestamp,
            update_time: timestamp
        };

        self.order_map.insert(order_id.borrow(), new_order.borrow());
        let mut sell_list = self.sell_order_map.get(previous_owner_id.borrow()).unwrap_or(Vec::new());
        sell_list.push(order_id);
        self.sell_order_map.insert(previous_owner_id.borrow(), sell_list.as_ref());

        let (ship_type, ship_subtype) = self.internal_get_ship_type_subtype_by_token_id(token_id.clone());
        
        // emit SellEvent
        Event::SellEvent{
            order_id, 
            previous_owner_id: &previous_owner_id, 
            token_id: &token_id.clone(), 
            order_status: OrderStatus::OrderSell as u8, 
            amount: U128(amount), 
            timestamp, 
            ship_type,
            ship_subtype
        }.emit();

        PromiseOrValue::Value(false)
    }

    /// 购买飞船
    pub fn internal_buy_spaceship(&mut self, sender_id: AccountId, order_id: u64, amount: U128) -> PromiseOrValue<bool> {
        let order = self.order_map.get(order_id.borrow());
        require!(order.is_some(), "Order does not exist");

        let mut order = order.unwrap();
        require!(order.status == OrderStatus::OrderSell as u8, "Market: invalid order status");
        require!(order.amount == amount.0, "invalid price");

        order.buyer = sender_id.clone();
        order.update_time = nano_to_sec(env::block_timestamp());
        order.status = OrderStatus::OrderBuy as u8;
        self.order_map.insert(order_id.borrow(), order.borrow());

        // settle amount
        let mut receiver_ids: Vec<String> = vec![];
        let mut amounts: Vec<U128> = vec![];

        let fee = order.amount as u128 * self.fee_rate as u128 / RATE_DENOMINATOR as u128;

        receiver_ids.push(order.seller.to_string());
        amounts.push(U128(fee));

        // empty address
        let burn = order.amount - fee;
        receiver_ids.push("".to_string());
        amounts.push(U128(burn));

        //how if seller is not registered at tia token
        // TODO. 
        ext_tokentia::batch_transfer(
            receiver_ids.clone(),
            amounts,
            None,
            self.token_tia.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER
        );

        // nft transfer 将spaceship发送给购买者
        ext_spaceship::nft_transfer(
            sender_id.clone(), 
            order.token_id, 
            None, 
            None, 
            self.spaceship.clone(), 
            1, 
            GAS_FOR_NFT_TRANSFER
        );

        // store user buy history
        let mut buy_list = self.buy_order_map.get(sender_id.borrow()).unwrap_or(Vec::new());
        buy_list.push(order_id);
        self.buy_order_map.insert(sender_id.borrow(), buy_list.as_ref());

        // emit BuyEvent
        Event::BuyEvent{
            order_id, 
            sender_id: &sender_id.clone(), 
            order_status: OrderStatus::OrderBuy as u8, 
            amount,
            fee: U128(fee), 
            update_time: order.update_time
        }.emit();

        PromiseOrValue::Value(true) // TODO 待修正
    }

    /// upgrade
    pub fn internal_upgrade_spaceship(&mut self, sender_id: AccountId, token_id_1: TokenId, token_id_2: TokenId, sub_type: u8, amount: U128) {
        let mut target_sub_type = sub_type;
 
        let (ship1_type, _) = self.internal_get_ship_type_subtype_by_token_id(token_id_1.clone());
        let (ship2_type, _) = self.internal_get_ship_type_subtype_by_token_id(token_id_2.clone());
        require!(ship1_type == ship2_type, "ShipFactory: material is must same type");

        require!(ship1_type == TYPE_B|| ship1_type == TYPE_C || ship1_type == TYPE_D, "ShipFactory: material is not allow type");

        let temp_mint: Vec<u128>;
        if target_sub_type == 0 {
            temp_mint = self.no_target_mint_fee.get(ship1_type.borrow()).unwrap();
        } else {
            let target_sub_type_limit = self.target_sub_type.get(ship1_type.borrow()).unwrap();
            require!( target_sub_type >= *target_sub_type_limit.get(0).unwrap() && target_sub_type <= *target_sub_type_limit.get(1).unwrap(), "Invalid ship subtype");
            temp_mint = self.target_mint_fee.get(ship1_type.borrow()).unwrap();
        }
        let tia_fee: u128 = *temp_mint.get(0).unwrap();
        let eng_fee: u128 = *temp_mint.get(1).unwrap();

        require!(amount.0 == tia_fee, "ShipFactory: invalid amount paid");

        // 批量转 start
        let ship_reward = tia_fee * self.ship_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let risker_reward = tia_fee * self.risker_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let bank_reward = tia_fee * self.bank_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let luck_reward = tia_fee * self.luck_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let rank_reward = tia_fee * self.rank_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let tia_burn = tia_fee - ship_reward - risker_reward - bank_reward - luck_reward - rank_reward;

        let mut receiver_ids: Vec<String> = vec![];
        let mut amounts: Vec<U128> = vec![];

        // ship pool reward
        receiver_ids.push(self.ship_pool.to_string());
        amounts.push(U128(ship_reward));

        // bank reward
        receiver_ids.push(self.bank.to_string());
        amounts.push(U128(bank_reward));

        // empty address 待修改为 SSP_UPGRADE_BURNED_ADDR
        receiver_ids.push("".to_string());
        amounts.push(U128(tia_burn));

        ext_tokentia::batch_transfer(
            receiver_ids.clone(),
            amounts,
            None,
            self.token_tia.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER
        );
        // 批量转 end

        // handle riskerpool
        let mut msg: String = String::from("");
        //
        ext_fungible_token::ft_transfer_call(
            self.risker_pool.clone(),
            U128(risker_reward),
            None,
            msg,
            self.token_tia.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER_CALL
        );
        //

        // handle rankPool
        //
        msg = String::from("{\"num\": ");
        msg += &"0".to_string();
        msg += &String::from("}");

        ext_fungible_token::ft_transfer_call(
            self.rank_pool.clone(),
            U128(rank_reward),
            None,
            msg,
            self.token_tia.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER_CALL
        );
        //

        // handle luckpool
        //
        msg = String::from("");
        ext_fungible_token::ft_transfer_call(
            self.luck_pool.clone(),
            U128(luck_reward),
            None,
            msg,
            self.token_tia.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER_CALL
        );
        //

        if target_sub_type == 0 {
            target_sub_type = self.internal_random_spaceship_subtype(ship1_type);
        }

        ext_spaceship::upgrade_spaceship(
            sender_id, 
            token_id_1, 
            token_id_2, 
            target_sub_type, 
            U128(eng_fee), 
            self.spaceship.clone(), 
            1, 
            GAS_FOR_SPACESHIP_UPGRADE
        );

    }
}

impl Contract {
    /// 通过token_id获取类型信息
    /// * `token_id`: 待查询的token_id
    pub fn internal_get_ship_type_subtype_by_token_id( &self, token_id: TokenId ) -> (u8, u8) {
        let items :Vec<&str> = token_id.split(":").collect();
        let ship_type: u8 = items[1].parse::<u8>().unwrap();
        let ship_subtype: u8 = items[2].parse::<u8>().unwrap();
        (ship_type, ship_subtype)
    }

    pub fn internal_random_spaceship_subtype(&mut self, ship_type: u8 ) -> u8{
        let mut sub_type: u8 = 0;
        let rnd: u64 = self.random();
        if ship_type == TYPE_A {
            sub_type = 1 + (rnd % 4) as u8; // 4 subtypes for A
        }
        if ship_type == TYPE_B {
            sub_type = 1 + (rnd % 8) as u8; // 8 subtypes for B
        }
        if ship_type == TYPE_C {
            sub_type = 1 + (rnd % 16) as u8; // 16 subtypes for C
        }
        if ship_type == TYPE_D {
            sub_type = 1 + (rnd % 32) as u8; // 32 subtypes for D
        }
        sub_type
    }

    pub fn random( &mut self ) -> u64 {
        // get random data
        let seeds: Vec<u8> = env::random_seed();
        let mut seed: u128 = 0;

        if self.shift > 24 {
            self.shift = 0;
        }

        for i in self.shift..self.shift+8{
            seed = seed * 10 + seeds[i] as u128;
        }
        self.shift += 2;

        let mut r = StdRng::seed_from_u64(seed as u64);
        let rnd: u64 = r.gen();

        rnd
    }
}

#[cfg(test)]
mod tests {

    use near_sdk::{AccountId, Gas};
    use crate::{Contract, OrderStatus};
    use near_sdk::{VMContext};

    fn get_default_context(_view_call: bool) -> VMContext {

        VMContext {
            current_account_id: AccountId::new_unchecked("alice_near".to_string()),
            signer_account_id: AccountId::new_unchecked("bob_near".to_string()),
            signer_account_pk: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap(),
            predecessor_account_id: AccountId::new_unchecked("carol_near".to_string()),
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: Gas(10u64.pow(18)),
            random_seed: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31],
            output_data_receivers: vec![],
            epoch_height: 0,
            view_config: None
        }
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_contract_sell() {
        let owner_account = AccountId::new_unchecked("owner".to_string());
        let ship_account = AccountId::new_unchecked("ship_account".to_string());
        let ssp_account = AccountId::new_unchecked("ssp_account".to_string());
        let ship_pool_account = AccountId::new_unchecked("ship_pool_account".to_string());
        let bank_account = AccountId::new_unchecked("bank_account".to_string());
        let risker_pool_account = AccountId::new_unchecked("risker_pool_account".to_string());
        let rank_pool_account = AccountId::new_unchecked("rank_pool_account".to_string());
        let luck_account = AccountId::new_unchecked("luck_account".to_string());
        let previous_owner_id = AccountId::new_unchecked("bob.near".to_string());

        let token_id = String::from("1:4:1:25");
        let msg = "20";

        let mut contract = Contract::new(owner_account, ship_account, ssp_account, ship_pool_account,
                                         bank_account, risker_pool_account, rank_pool_account, luck_account);
        let _result = contract.internal_sell_spaceship(previous_owner_id, token_id, msg.to_string());

        // assert section
        let previous_owner_id = AccountId::new_unchecked("bob.near".to_string());
        let sell_list = contract.sell_order_map.get(&previous_owner_id).unwrap_or(Vec::new());

        assert_eq!(sell_list.len(), 1, "Quantity mismatch");
    }

    #[test]
    fn test_contract_cancel() {
        get_default_context(false);
        // 下面到第一个assert部分与sell相同，便于后面取消使用
        let owner_account = AccountId::new_unchecked("owner".to_string());
        let ship_account = AccountId::new_unchecked("ship_account".to_string());
        let ssp_account = AccountId::new_unchecked("ssp_account".to_string());
        let ship_pool_account = AccountId::new_unchecked("ship_pool_account".to_string());
        let bank_account = AccountId::new_unchecked("bank_account".to_string());
        let risker_pool_account = AccountId::new_unchecked("risker_pool_account".to_string());
        let rank_pool_account = AccountId::new_unchecked("rank_pool_account".to_string());
        let luck_account = AccountId::new_unchecked("luck_account".to_string());

        let previous_owner_id = AccountId::new_unchecked("bob.near".to_string());

        let token_id = String::from("1:4:1:25");
        let msg = "20";

        let mut contract = Contract::new(owner_account, ship_account, ssp_account, ship_pool_account,
                                         bank_account, risker_pool_account, rank_pool_account, luck_account);
        let _result = contract.internal_sell_spaceship(previous_owner_id, token_id, msg.to_string());

        // assert section
        let previous_owner_id = AccountId::new_unchecked("bob.near".to_string());
        let sell_list = contract.sell_order_map.get(&previous_owner_id).unwrap_or(Vec::new());

        assert_eq!(sell_list.len(), 1, "Quantity mismatch");

        contract.user_cancel_sell_spaceship(1);

        let order = contract.order_map.get(&1).unwrap();
        assert_eq!(order.status, OrderStatus::OrderCancel as u8, "Status Error");
    }
}
