use near_contract_standards::non_fungible_token::TokenId;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, PanicOnDefault,
    PromiseOrValue, Gas, ext_contract, log, Balance
};


mod view;
mod owner;
mod utils;
mod events;

pub use crate::utils::*;
pub use crate::events::*;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_TRANSFER: Gas = Gas(30 * TGAS);
pub const GAS_FOR_TRANSFER_ON_CALL: Gas = Gas(45 * TGAS);

pub const YOCTO18: u128 = 1_000_000_000_000_000_000;
pub const ONE_DAY_IN_SECS: u64 = 24 * 60 * 60;
pub const RATE_DENOMINATOR: u128 = 100;
pub const DEFAULT_REBATE_RATE: u8 = 10;
pub const DEFAULT_DURATION_SEC: u64 = 60 * 60;
pub const MAX_DURATION_SEC: u64 = 72 * 60 * 60;

pub type PriceType = u128;
pub type TimeStampSec = u64;
pub type AuctionId = u64;


#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct AuctionInfo {
    pub buyer: AccountId,
    pub token_id: TokenId,
    #[serde(with = "u128_dec_format")]
    pub price: u128,
    #[serde(with = "u128_dec_format")]
    pub up_price: u128,
    pub start: TimeStampSec,
    pub end:   TimeStampSec,
    #[serde(with = "u128_dec_format")]
    pub team_fund: u128,
    pub claimed: bool,
    pub team_fund_claimed: bool,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,

    // NFT address
    spaceship_id: AccountId,
    // token address
    token_id: AccountId,
    // team id;
    team_id: AccountId,

    rebate_rate: u8, // default: 10;
    duration_sec: u64, // default: 60*60,  1 hours;

    pool: Vector<AuctionInfo>,

    balance: Balance, // reserved
}

#[ext_contract(ext_spaceship)]
pub trait spaceship {
    fn nft_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
    );
}


#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, spaceship_id: AccountId, token_id: AccountId, team_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id,
            spaceship_id,
            token_id,
            team_id,
            rebate_rate: DEFAULT_REBATE_RATE,
            duration_sec: DEFAULT_DURATION_SEC,
            pool: Vector::new(b"v".to_vec()),
            balance: 0,
        }
    }

    /* ========== CORE FUNCTION ========== */
    // AUC_O0_03
    pub fn claim(&mut self, auction_id: AuctionId ) {
        let predecessor_id = env::predecessor_account_id();
        let mut auction_info: AuctionInfo = self.pool.get(auction_id).expect("invalid auction_id");
        require!(auction_info.end < nano_to_sec(env::block_timestamp()), "ERR_AUCTION_STILL_RUNNING");
        require!( predecessor_id == auction_info.buyer, "ERR_NOT_AUCTION_WINNER");
        require!(!auction_info.claimed, "ERR_AUCTION_ALREADY_CLAIMED");

        ext_spaceship:: nft_payout( 
            auction_info.buyer.clone(),
            auction_info.token_id.clone(), 
            self.spaceship_id.clone(),
            0,
            GAS_FOR_TRANSFER 
        );

        // update auction_info info
        if !auction_info.team_fund_claimed {
            ext_fungible_token::ft_transfer(
                self.team_id.clone(), 
                U128(auction_info.team_fund),
                None,
                self.token_id.clone(),
                1,
                GAS_FOR_TRANSFER            
            ); 
            auction_info.team_fund_claimed = true;
        }

        auction_info.claimed = true;
        self.pool.replace(auction_id, &auction_info);
    }


}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// AUC_O0_01
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let predecessor_id = env::predecessor_account_id();
        require!( predecessor_id == self.token_id, "invalid token contract id");
        let auction_id: AuctionId = msg.parse::<AuctionId>().expect("msg must contain all digits");
        self.internal_bid( sender_id.clone(), auction_id, amount.0);

        Event::Buy{buyer_id: &sender_id, auction_id, token_id: &predecessor_id.to_string(),price: &amount}.emit();

        PromiseOrValue::Value(U128(0))
    }
}

impl Contract{
    // AUC_O0_02
    pub fn internal_bid(&mut self, buyer: AccountId, auction_id: AuctionId, amount: Balance) {
        let mut auction_info: AuctionInfo = self.pool.get(auction_id).expect("invalid auction_id");
        require!(auction_info.start <= nano_to_sec(env::block_timestamp()), "ERR_AUCTION_NOT_START");
        require!(auction_info.end >= nano_to_sec(env::block_timestamp()), "ERR_AUCTION_ENDED");

        let (price, rebate, refund, team_fund) = 
        if auction_info.buyer == self.owner_id {
            // check first buy
            (auction_info.price, 0, auction_info.price, auction_info.price)

        } else {
            let last_price = auction_info.price;
            let rebate = auction_info.up_price * self.rebate_rate as u128 / RATE_DENOMINATOR;
            let refund = last_price + rebate;
            let price = last_price + auction_info.up_price;
            let team_fund = last_price + price - refund;
            (price, rebate, refund, team_fund)
        };

        require!( amount == price, "ERR_INVALID_PRICE");

        // refund and rebate
        if rebate > 0 {
            ext_fungible_token::ft_transfer(
                auction_info.buyer,
                U128(refund),
                None,
                self.token_id.clone(),
                1,
                GAS_FOR_TRANSFER            
            ); 
        }

        // update auction_info info
        auction_info.buyer = buyer;
        auction_info.price = price;
        auction_info.team_fund = team_fund;
        auction_info.end = std::cmp::min(
            auction_info.end + self.duration_sec, 
            nano_to_sec(env::block_timestamp()) + MAX_DURATION_SEC,
        );
        self.pool.replace(auction_id, &auction_info);

    }
}
