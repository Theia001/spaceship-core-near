#![allow(dead_code)]

use near_sdk::{AccountId, Balance, Gas};
pub use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};

pub use token_tia::{ContractContract as TokenContract, Metadata};


// mod owner;
// pub use owner::*;


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_WASM_BYTES => "../../res/token_tia.wasm",
    CUR_WASM_BYTES => "../../res/token_tia.wasm",
}

pub fn previous_wasm_bytes() -> &'static [u8] {
    &PREV_WASM_BYTES
}

pub fn cur_wasm_bytes() -> &'static [u8] {
    &CUR_WASM_BYTES
}


pub const NEAR: &str = "near";
pub const TOKEN_TIA_ID: &str = "tia.near";
pub const OWNER_ID: &str = "owner.near";

pub const DEFAULT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);
pub const TOKEN_DECIMALS: u8 = 18;
pub const TOKEN_TOTAL_SUPPLY: Balance =
    100_000_000 * 10u128.pow(TOKEN_DECIMALS as _);

pub struct Env {
    pub root: UserAccount,
    pub near: UserAccount,
    pub owner: UserAccount,
    pub token_contract: ContractAccount<TokenContract>,
}

pub fn init_env() -> Env {
    Env::init_with_contract(&CUR_WASM_BYTES)
}

impl Env {
    pub fn init_with_contract(contract_bytes: &[u8]) -> Self {
        
        let root = init_simulator(None);
        let near = root.create_user(
            AccountId::new_unchecked(NEAR.to_string()),
            to_yocto("100000"),
        );
        let owner = near.create_user(
            AccountId::new_unchecked(OWNER_ID.to_string()),
            to_yocto("1000"),
        );

        let token_contract = deploy!(
            contract: TokenContract,
            contract_id: TOKEN_TIA_ID.to_string(),
            bytes: &contract_bytes,
            signer_account: near,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS.0,
            init_method: new(
                owner.account_id(),
                "name".to_string(),
                "symbol".to_string(),
                18
            )
        );

        Self {
            root,
            near,
            owner,
            token_contract,
        }
    }

    pub fn upgrade_contract(&self, user: &UserAccount, contract_bytes: &[u8]) -> ExecutionResult {
        user
            .create_transaction(account_id(TOKEN_TIA_ID))
            .function_call("upgrade".to_string(), contract_bytes.to_vec(), MAX_GAS.0, 0)
            .submit()
    }

    pub fn get_metadata(&self) -> Metadata{
        self.owner
        .view_method_call(
            self.token_contract.contract.get_metadata()
        ).unwrap_json::<Metadata>()
    }

}

pub struct Users {
    pub alice: UserAccount,
    pub bob: UserAccount,
    pub charlie: UserAccount,
}

impl Users {
    pub fn init(e: &Env) -> Self {
        Self {
            alice: e.near.create_user(account_id("alice.near"), to_yocto("10000")),
            bob: e.near.create_user(account_id("bob.near"), to_yocto("10000")),
            charlie: e.near.create_user(account_id("charlie.near"), to_yocto("10000")),
        }
    }
}

pub fn d(value: Balance, decimals: u8) -> Balance {
    value * 10u128.pow(decimals as _)
}
pub fn account_id(account_id: &str) -> AccountId {
    AccountId::new_unchecked(account_id.to_string())
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