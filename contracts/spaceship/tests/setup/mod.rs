#![allow(dead_code)]

pub use near_sdk::json_types::U128;
use near_sdk::{AccountId, Balance, Gas};
pub use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
pub use near_contract_standards::non_fungible_token::{Token, TokenId};
use mock_receiver::ContractContract as Mock;
use spaceship::{ContractContract as SpaceShip, Metadata, ShipElements, SpaceShipSupply};

mod views;
pub use views::*;
mod ft;
pub use ft::*;
mod nft;
pub use nft::*;
mod owner;
pub use owner::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    SPACESHIP_WASM_BYTES => "../../res/spaceship.wasm",
    PREV_SPACESHIP_WASM_BYTES => "../../res/spaceship.wasm",
    MOCK_WASM_BYTES => "../../res/mock_receiver.wasm",
}

pub const OWNER_ID: &str = "owner";
pub const MAGICBOX_ID: &str = "magicbox";
pub const SHIPMARKET_ID: &str = "shipmarket";
pub const AUCTION_ID: &str = "auction";
pub const LUCKPOOL_ID: &str = "luckpool";
pub const SPACESHIP_ID: &str = "spaceship";
pub const SHIPPOOL_ID: &str = "shippool";

pub const DEFAULT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);
pub const TOKEN_DECIMALS: u8 = 24;
pub const TOKEN_TOTAL_SUPPLY: Balance = 1_000_000_000 * 10u128.pow(TOKEN_DECIMALS as _);

pub fn previous_spaceship_wasm_bytes() -> &'static [u8] {
    &PREV_SPACESHIP_WASM_BYTES
}

pub fn spaceship_wasm_bytes() -> &'static [u8] {
    &SPACESHIP_WASM_BYTES
}

pub struct Env {
    pub root: UserAccount,
    pub owner: UserAccount,
    pub magicbox: UserAccount,
    pub shipmarket: ContractAccount<Mock>,
    pub auction: UserAccount,
    pub luckpool: UserAccount,
    pub spaceship: ContractAccount<SpaceShip>,
    pub shippool: ContractAccount<Mock>,
}

impl Env {
    pub fn init_with_contract(contract_bytes: &[u8]) -> Self {
        let root = init_simulator(None);

        let owner = root.create_user(OWNER_ID.parse().unwrap(), to_yocto("100"));
        let magicbox = root.create_user(MAGICBOX_ID.parse().unwrap(), to_yocto("100"));
        let auction = root.create_user(AUCTION_ID.parse().unwrap(), to_yocto("100"));
        let luckpool = root.create_user(LUCKPOOL_ID.parse().unwrap(), to_yocto("100"));

        let spaceship = deploy!(
            contract: SpaceShip,
            contract_id: SPACESHIP_ID.to_string(),
            bytes: contract_bytes,
            signer_account: root,
            deposit: to_yocto("100"), // Deposit required to cover contract storage.
            gas: near_sdk_sim::DEFAULT_GAS,
            init_method: new(owner.account_id(), magicbox.account_id(), SHIPPOOL_ID.parse().unwrap(), SHIPMARKET_ID.parse().unwrap(), auction.account_id(), luckpool.account_id())
        );

        let shippool = deploy!(
            contract: Mock,
            contract_id: SHIPPOOL_ID.to_string(),
            bytes: &MOCK_WASM_BYTES,
            signer_account: root,
            init_method: new(spaceship.account_id())
        );

        let shipmarket = deploy!(
            contract: Mock,
            contract_id: SHIPMARKET_ID.to_string(),
            bytes: &MOCK_WASM_BYTES,
            signer_account: root,
            init_method: new(spaceship.account_id())
        );

        Self {
            root,
            owner,
            magicbox,
            shipmarket,
            auction,
            luckpool,
            spaceship,
            shippool,
        }
    }

    pub fn upgrade_contract(&self, user: &UserAccount, contract_bytes: &[u8]) -> ExecutionResult {
        user.create_transaction(SPACESHIP_ID.parse().unwrap())
            .function_call("upgrade".to_string(), contract_bytes.to_vec(), MAX_GAS.0, 0)
            .submit()
    }
}

pub fn init_env() -> Env {
    Env::init_with_contract(&SPACESHIP_WASM_BYTES)
}

pub fn show_promises(r: &ExecutionResult) {
    for promise in r.promise_results() {
        println!("{:?}", promise);
    }
}

pub fn total_consumed_gas(r: &ExecutionResult) -> u64 {
    let mut ret = 0_u64;
    for promise in r.promise_results() {
        if let Some(outcome) = promise {
            ret += outcome.gas_burnt().0;
        }
    }
    ret
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
macro_rules! assert_err {
    (print $exec_func: expr) => {
        println!(
            "{:?}",
            $exec_func.promise_errors()[0].as_ref().unwrap().status()
        );
    };
    ($exec_func: expr, $err_info: expr) => {
        assert!(format!(
            "{:?}",
            $exec_func.promise_errors()[0].as_ref().unwrap().status()
        )
        .contains($err_info));
    };
}
