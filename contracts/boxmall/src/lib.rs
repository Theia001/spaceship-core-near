use near_contract_standards::non_fungible_token::{TokenId};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet, LookupMap, UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
    PromiseOrValue, Gas, ext_contract, Promise, log, Balance, serde_json
};
use rand_distr::{Normal, Distribution};
use rand::{Rng,SeedableRng};
use rand::rngs::StdRng;

mod view;
mod owner;
mod invite;
mod utils;
mod events;

pub use crate::utils::*;
pub use crate::events::*;

pub type BoxType = u8;
pub type PriceType = u128;
pub type TimeStampSec = u64;

// box type
pub const TYPE_U: u8 = 1;
pub const TYPE_S: u8 = 2;
// ship type
pub const TYPE_SHIP_A: u8 = 4;
pub const TYPE_SHIP_B: u8 = 3;
pub const TYPE_SHIP_C: u8 = 2;
pub const TYPE_SHIP_D: u8 = 1;
// reward basic
pub const RATE_DENOMINATOR: u8 = 100;
// prob basic
pub const PROB_DENOMINATOR: u64 = 10000;

pub const YOCTO18: u128 = 1_000_000_000_000_000_000;

pub const SHIP_REWARD_RATE: u8 = 25;
pub const RISKER_REWARD_RATE: u8 = 5;
pub const BANK_REWARD_RATE: u8 = 5;
pub const RANK_REWARD_RATE: u8 = 5;
pub const INVITE_REWARD_RATE: u8 = 5;
pub const LUCK_REWARD_RATE: u8 = 5;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SBoxPriceInfo{
    // sBoxPrice
    sbox_price: u128, // default: 30e18,
    // sspTWapPriceMin
    ssp_twap_price_min: u128, // default: 10e18,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardRate{
    // shipRewardRate
    ship_reward_rate: u8, // default: 25,
    // riskerRewardRate
    risker_reward_rate: u8, // default: 5,
    // bankRewardRate
    bank_reward_rate: u8, // default: 5,
    // rankRewardRate
    rank_reward_rate: u8, // default: 5,
    // inviteRewardRate
    invite_reward_rate: u8, // default: 5,
    // luckRewardRate
    luck_reward_rate: u8,// default: 5,
}

#[derive( BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct UBoxSale {
    total: u32,
    price: PriceType,
    sale: u32,
    start: TimeStampSec,
    end: TimeStampSec,
}

/*
// this struct is used to pass parameter for BuyU/BuyS
{
  "type": "buy_u",
  "num": 10,
}
*/

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TransferCallInfo {
   BuyInfo{ box_type: String, num: u32 },
   ContractCallInfo{ contract_id: String, user_id: AccountId },
}

// invite.sol related
#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Children {
    addr: AccountId,
    create_time: TimeStampSec,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub struct Relation {
    parent: String, // AccountId can not be set to ""
    #[serde(with = "u128_dec_format")]
    donate: u128,
    #[serde(with = "u128_dec_format")]
    donate_u: u128,
    num: u32,
    children: Vec<Children>,
}
// end of invite.sol related

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_BATCH_TRANSFER: Gas = Gas(25 * TGAS);
pub const GAS_FOR_BATCH_TRANSFER_CALL: Gas = Gas(50 * TGAS);
pub const GAS_FOR_BATCH_MINT_BOX: Gas = Gas(70 * TGAS);
#[ext_contract(ext_magicbox)]
pub trait magicbox {
    fn batch_mint(&mut self, 
        token_owner_id: AccountId,
        box_type: BoxType,
        num: u32,
    );
}

#[ext_contract(ext_tokentia)]
pub trait tokentia{
    fn batch_transfer(
        &mut self,
        receiver_ids: Vec<String>,
        amounts: Vec<U128>,
        memo: Option<String>,
    );

    fn batch_transfer_call(
        &mut self,
        receiver_ids: Vec<String>,
        amounts: Vec<U128>,
        msgs: Vec<String>,
        memo: Option<String>,
    ) -> PromiseOrValue<U128>;
}
#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    TokensPerOwner { account_hash: Vec<u8> },
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    HolderTypeTokens,
    TokenIdToType,
    TypeBalance,
    TypeBurnBalance,
    TypeProb,
    // consolidate the function of shipwallet contract
    Balances,
    // consolidate the function of invite contract
    UserRelation, 
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    
    // uBoxSalePool
    ubox_sale_pool: Vector<UBoxSale>,
    
    magicbox: AccountId,
    usn: AccountId,
    token_tia: AccountId,
    bank: AccountId,
    // bankU
    bank_u: AccountId,
    oracle: AccountId,
    risker_pool: AccountId,
    rank_pool: AccountId,
    //spaceship: AccountId,
    ship_pool: AccountId,
    //invite: AccountId,
    //ship_wallet: AccountId,
    luck: AccountId,
    
    sbox_price_info: SBoxPriceInfo,
    // misc reward rate
    reward_rate: RewardRate,

    // buyUSwitch
    buy_u_switch: bool,// = true;
    // buySSwitch
    buy_s_switch: bool,// = false;
    // sPriceSwitch
    s_price_switch: bool,// = true;

    // numLimit
    num_limit: u8, // = 10,
    // uBoxSaleNumLimit
    ubox_sale_num_limit: u32,// = 10000;

    // sale data
    // uBoxSaleNum
    ubox_sale_num: u32,
    // uBoxSaleAmount
    ubox_sale_amount: u128,
    // sBoxSaleNum
    sbox_sale_num: u32,
    // sBoxSaleAmount
    sbox_sale_amount: u128,

    // consolidate the function from shipwallet.sol
    //key is AccountId, return balance
    balances: UnorderedMap<AccountId, Balance>,
    // _total_supply
    total_balance_shipwallet: Balance,

    // consolidate the function from invite.sol
    //ssp: AccountId, // spaceshiptoken
    //ship_wallet_contract_id: AccountId,
    //usn: AccountId,
    user_relation: UnorderedMap<AccountId, Relation>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, magicbox: AccountId, usn: AccountId, token_tia: AccountId,
                bank: AccountId, bank_u: AccountId, oracle: AccountId, 
                risker_pool: AccountId,
                rank_pool: AccountId, ship_pool: AccountId, //spaceship: AccountId, invite: AccountId,
                //ship_wallet: AccountId, 
                luck: AccountId 
                ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id,

            ubox_sale_pool: Vector::new(b"v".to_vec()),
            
            magicbox,
            usn,
            token_tia,
            bank,
            bank_u,
            oracle,
            risker_pool,
            rank_pool,
            ship_pool,
            //spaceship,
            //invite,
            //ship_wallet,
            luck,
            
            sbox_price_info: SBoxPriceInfo{
                sbox_price: 30 * YOCTO18, // default: 30
                ssp_twap_price_min: YOCTO18,
            },

            reward_rate: RewardRate{
                ship_reward_rate: SHIP_REWARD_RATE,
                risker_reward_rate: RISKER_REWARD_RATE,
                bank_reward_rate: BANK_REWARD_RATE,
                rank_reward_rate: RANK_REWARD_RATE,
                invite_reward_rate: INVITE_REWARD_RATE,
                luck_reward_rate: LUCK_REWARD_RATE,                
            },

            buy_u_switch:  true,
            buy_s_switch:  true,
            s_price_switch:  true,
            num_limit:  5, // default: 10. Currently set to 5. keep same value as magicbox
            ubox_sale_num_limit: 10000,        
            
            // sale data
            ubox_sale_num: 0,
            ubox_sale_amount: 0,
            sbox_sale_num: 0,
            sbox_sale_amount: 0,

            balances: UnorderedMap::new( StorageKey::Balances),
            total_balance_shipwallet: 0,
            user_relation: UnorderedMap::new(StorageKey::UserRelation),
        }
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// BML-00-01
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.usn || predecessor_id == self.token_tia , "Invalid contract Id");

        let mut refund: u128 = 0;

        let info: TransferCallInfo = serde_json::from_str::<TransferCallInfo>(&msg).expect("invalid msg");

        match info {
            TransferCallInfo::BuyInfo{box_type, num} => {
                if box_type == "buy_u".to_string() {
                    refund = self.internal_buy_u(amount.clone(),sender_id.clone(), num );
                }
                if box_type == "buy_s".to_string() {
                    refund = self.internal_buy_s(amount.clone(),sender_id.clone(), num );
                }
            },
            TransferCallInfo::ContractCallInfo{contract_id, user_id} => {
                if  contract_id == "trialpool".to_string() ||
                    contract_id == "collectpool".to_string() ||
                    contract_id == "corepool".to_string() {
                 self.internal_ship_wallet_add(user_id.clone(),amount.0);
             }                
            },
         }

        PromiseOrValue::Value(U128(refund))
    }


}

impl Contract {
   
    /* ========== CORE FUNCTION ========== */
    // -->USN-->BOXMALL-->
    // USN contract (from user, to BoxMall). boxmall
    // BoxMall. ft_transfer_call
    //
    // msg->{"type": "buy_u","num": 10}
    //
    // BML-00-02
    pub fn internal_buy_u(&mut self, amount: U128, buyer_id:AccountId, num: u32) -> u128 {
        
        let mut refund: u128 = 0;
        let actual_payamount: u128 = amount.0;
       
        self.ubox_sale_num += num; // use temp variable

        let sale_pool_len = self.ubox_sale_pool.len();
        // conditions check
        require!( self.ubox_sale_num <= self.ubox_sale_num_limit, "num: is over");
        require!( sale_pool_len > 0, "ubox_sale_pool is empty");
        require!( self.buy_u_switch == true, "buy_u_swith is false");
        // check ubox_switch
        // conditions check

        let mut sale: UBoxSale = self.ubox_sale_pool.get(sale_pool_len-1).unwrap().clone();
        sale.sale += num;
        let u_box_price = sale.price;
        
        let mut totalamount: u128 = num as u128 *u_box_price;
        require!(totalamount <= actual_payamount, "payment less than actual amount");
        refund = actual_payamount - totalamount;

        
        self.ubox_sale_amount += totalamount;
        
        // transfer amount. Done by user already!
        //IERC20(usn).safeTransferFrom(msg.sender, address(this), amount);
        
        let invite_reward: u128 = totalamount * self.reward_rate.invite_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let relation: Relation = self.get_info(buyer_id.clone());
        if relation.parent != "".to_string() {
            self.reward_u(buyer_id.clone(), relation.parent.parse().unwrap(), invite_reward);
            totalamount -= invite_reward;
        }

        // transfer amount to USN account
        //IERC20(usn).safeTransfer(bankU, amount);
        ext_fungible_token::ft_transfer(
            self.bank_u.clone(),
            U128(totalamount),
            None,
            self.owner_id.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER
        );

        // mint num boxes
        ext_magicbox::batch_mint(buyer_id.clone(),
            TYPE_U,
            num,
            self.magicbox.clone(),
            1,
            GAS_FOR_BATCH_MINT_BOX
        );
        self.ubox_sale_pool.replace(sale_pool_len-1, &sale);
        Event::BuyU{caller_id: &env::predecessor_account_id(), buyer_id: &buyer_id, amount: &amount, num}.emit();
        refund
        
    }
    
    
    // -->TIA-->BOXMALL-->
    //
    // msg->{"type": "buy_s","num": 10}
    //
    // BML-00-03
    pub fn internal_buy_s(&mut self, amount: U128, buyer_id:AccountId, num: u32) -> u128 {
        let mut refund: u128 = 0;
        let actual_payamount: u128 = amount.0;
        let ssp_price: u128 = self.get_box_ssp_price();
        let totalamount: u128 = ssp_price*num as u128;
        let balance: u128 = self.ship_wallet_balance_of(buyer_id.clone());

        // conditions check
        // 
        require!( self.buy_s_switch == true, "buy_s_swith is false");
        require!(totalamount <= actual_payamount + balance,"Error: totalamount more than (payment + balance)");
        // end conditions check

        if totalamount <= balance {
            self.internal_ship_wallet_sub(buyer_id.clone(), totalamount);

            refund = actual_payamount;
        }
        else{
            self.internal_ship_wallet_sub(buyer_id.clone(), balance);

            refund = actual_payamount - ( totalamount - balance );
        }

        let mut ship_reward: u128 = totalamount * self.reward_rate.ship_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let risker_reward: u128 = totalamount * self.reward_rate.risker_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let bank_reward: u128 = totalamount * self.reward_rate.bank_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let rank_reward: u128 = totalamount * self.reward_rate.rank_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let invite_reward: u128 = totalamount * self.reward_rate.invite_reward_rate as u128 / RATE_DENOMINATOR as u128;
        let luck_reward: u128 = totalamount * self.reward_rate.luck_reward_rate as u128 / RATE_DENOMINATOR as u128;

        let sum = ship_reward + risker_reward + luck_reward + bank_reward + rank_reward + invite_reward;
        require!(totalamount >= sum,"error: totalamount < sum");
        let burn: u128 = totalamount- sum;


        // declare here
        let mut receiver_ids: Vec<String> = vec![];
        let mut amounts: Vec<U128> = vec![];


        let relation: Relation = self.get_info(buyer_id.clone());
        if relation.parent != "".to_string() {
            self.reward_token(buyer_id.clone(), relation.parent.parse().unwrap(), invite_reward);
            
            // transfer to relation.parent in batch to save gas
            receiver_ids.push(relation.parent.clone());
            amounts.push(U128(invite_reward)); 
        }
        else {
            ship_reward += invite_reward;
        }

        receiver_ids.push(self.luck.clone().to_string());
        amounts.push(U128(luck_reward));

        receiver_ids.push(self.ship_pool.clone().to_string());
        amounts.push(U128(ship_reward));
  
        receiver_ids.push(self.bank.clone().to_string());
        amounts.push(U128(bank_reward));

        // empty address
        receiver_ids.push("".to_string());
        amounts.push(U128(burn));

        // /// Arguments:
        // /// - `receiver_ids` - each receivers account ID, an empty string means burn token.
        // /// - `amounts` - the amount of tokens to each receiver_id.
        // /// - `memo` - a string message that was passed with this transfer, will be recorded as log
        // pub fn batch_transfer(
        //     receiver_ids: Vec<String>,
        //     amounts: Vec<U128>,
        //     memo: Option<String>,
        // );
        ext_tokentia::batch_transfer(
            receiver_ids.clone(),
            amounts,
            None,
            self.token_tia.clone(),
            1,
            GAS_FOR_BATCH_TRANSFER
        );

        //
        // handle riskerpool
        let mut msg: String = String::from("{\"buyer_id\": \"");
        msg += &buyer_id.to_string();
        msg += &String::from("\"}");
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
        msg += &num.to_string();
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


        // mint box
        //
        ext_magicbox::batch_mint(buyer_id.clone(),
        TYPE_S,
            num,
            self.magicbox.clone(),
            0,
            GAS_FOR_BATCH_MINT_BOX
        );
        //

        Event::BuyS{caller_id: &env::predecessor_account_id(), buyer_id: &buyer_id, amount: &amount, num}.emit();

        refund
        
    }
    
    // shipwallet functions
    // BML-00-08
    pub fn internal_ship_wallet_add( &mut self, to: AccountId, amount: u128 ) {
        let mut balance = self.balances.get(&to).unwrap_or(0);
        balance += amount;
        self.balances.insert(&to, &balance);
        self.total_balance_shipwallet += amount;
    }
    // BML-00-09
    pub fn internal_ship_wallet_sub( &mut self, to: AccountId, amount: u128 ) {
        let mut balance = self.balances.get(&to).unwrap_or(0);
        balance -= amount;
        self.balances.insert(&to, &balance);
        self.total_balance_shipwallet -= amount;
    }

}