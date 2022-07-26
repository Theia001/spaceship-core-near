#![allow(dead_code)]

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::json_types::U128;
use near_sdk::{AccountId, Balance, Gas, Timestamp};
use near_sdk_sim::runtime::GenesisConfig;
pub use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};

use near_contract_standards::non_fungible_token::{Token, TokenId};

use auction::{ContractContract as Auction, AuctionInfo, AuctionId, TimeStampSec};
use mock_nft::ContractContract as Nft;
use mock_usn::ContractContract as Usn;

mod views;
pub use views::*;
mod owner;
pub use owner::*;
mod interface;
pub use interface::*;
mod usn;
pub use usn::*;
mod nft;
pub use nft::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    AUCTION_WASM_BYTES => "../../res/auction.wasm",
    PREV_AUCTION_WASM_BYTES => "../../res/auction.wasm",
    MOCKUSN_WASM_BYTES => "../../res/mock_usn.wasm",
    MOCKNFT_WASM_BYTES => "../../res/mock_nft.wasm",
}

pub const AUCTION_ID: &str = "auction";
pub const USN_ID: &str = "usn";
pub const FT_ID: &str = "ft";
pub const NFT_ID: &str = "nft";
pub const OWNER_ID: &str = "owner";
pub const TEAM_ID: &str = "team";

pub const DEFAULT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);
pub const TOKEN_DECIMALS: u8 = 24;
pub const TOKEN_TOTAL_SUPPLY: Balance = 1_000_000_000 * 10u128.pow(TOKEN_DECIMALS as _);

pub const GENESIS_TIMESTAMP: u64 = 100 * 10u64.pow(9);

pub fn previous_auction_wasm_bytes() -> &'static [u8] {
    &PREV_AUCTION_WASM_BYTES
}

pub fn auction_wasm_bytes() -> &'static [u8] {
    &AUCTION_WASM_BYTES
}

pub struct Env {
    pub root: UserAccount,
    pub owner: UserAccount,
    pub team: UserAccount,
    pub auction: ContractAccount<Auction>,
    pub usn: ContractAccount<Usn>,
    pub nft: ContractAccount<Nft>,
    pub ft: ContractAccount<Usn>,
}

impl Env {
    pub fn init_with_contract(contract_bytes: &[u8]) -> Self {
        let mut genesis_config = GenesisConfig::default();
        genesis_config.genesis_time = GENESIS_TIMESTAMP;
        genesis_config.block_prod_time = 0;
        
        let root = init_simulator(Some(genesis_config));

        let owner = root.create_user(OWNER_ID.parse().unwrap(), to_yocto("100"));
        let team = root.create_user(TEAM_ID.parse().unwrap(), to_yocto("100"));
        
        let auction = deploy!(
            contract: Auction,
            contract_id: AUCTION_ID.to_string(),
            bytes: contract_bytes,
            signer_account: root,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(owner.account_id(), NFT_ID.parse().unwrap(), USN_ID.parse().unwrap(), team.account_id())
        );
    
        let nft = deploy!(
            contract: Nft,
            contract_id: NFT_ID.to_string(),
            bytes: &MOCKNFT_WASM_BYTES,
            signer_account: root,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(owner.account_id())
        );
    
        let usn = deploy!(
            contract: Usn,
            contract_id: USN_ID.to_string(),
            bytes: &MOCKUSN_WASM_BYTES,
            signer_account: root,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new()
        );

        let ft = deploy!(
            contract: Usn,
            contract_id: FT_ID.to_string(),
            bytes: &MOCKUSN_WASM_BYTES,
            signer_account: root,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new()
        );

        Self {
            root,
            owner,
            team,
            auction,
            usn,
            nft,
            ft,
        }
    }

    pub fn upgrade_contract(&self, user: &UserAccount, contract_bytes: &[u8]) -> ExecutionResult {
        user
            .create_transaction(AUCTION_ID.parse().unwrap())
            .function_call("upgrade".to_string(), contract_bytes.to_vec(), MAX_GAS.0, 0)
            .submit()
    }

    pub fn skip_time(&self, seconds: u32) {
        self.root.borrow_runtime_mut().cur_block.block_timestamp += to_nano(seconds);
    }

    pub fn set_time(&self, seconds: u32) {
        assert!(to_nano(seconds) > self.current_time());
        self.root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(seconds);
    }

    pub fn current_time(&self) -> u64{
        self.root.borrow_runtime().cur_block.block_timestamp
    }
}

pub fn init_env() -> Env {
    Env::init_with_contract(&AUCTION_WASM_BYTES)
}

pub fn to_nano(timestamp: u32) -> Timestamp {
    Timestamp::from(timestamp) * 10u64.pow(9)
}

pub fn to_sec(timestamp: Timestamp) -> u32 {
    (timestamp / 10u64.pow(9)) as u32
}

pub fn show_promises(r: &ExecutionResult) {
    for promise in r.promise_results() {
        println!("{:?}", promise);
    }
}

pub fn get_logs(r: &ExecutionResult) -> Vec<String> {
    let mut logs: Vec<String> = vec![];
    r.promise_results()
        .iter()
        .map(|ex| {
            ex.as_ref()
                .unwrap()
                .logs()
                .iter()
                .map(|x| logs.push(x.clone()))
                .for_each(drop)
        })
        .for_each(drop);
    logs
}

pub fn get_error_count(r: &ExecutionResult) -> u32 {
    r.promise_errors().len() as u32
}

pub fn get_error_status(r: &ExecutionResult) -> String {
    format!("{:?}", r.promise_errors()[0].as_ref().unwrap().status())
}

#[macro_export]
macro_rules! assert_err{
    (print $exec_func: expr)=>{
        println!("{:?}", $exec_func.promise_errors()[0].as_ref().unwrap().status());
    };
    ($exec_func: expr, $err_info: expr)=>{
        assert!(format!("{:?}", $exec_func.promise_errors()[0].as_ref().unwrap().status()).contains($err_info));
    };
}