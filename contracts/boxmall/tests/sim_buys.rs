use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{env, log, near_bindgen, require, AccountId, PanicOnDefault};
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, ExecutionResult, UserAccount,
    DEFAULT_GAS,
};

use near_contract_standards::non_fungible_token::Token;

use boxmall::ContractContract as boxmall;
use mock_usn::ContractContract as mock_usn;
use magicbox::ContractContract as magicbox;
use spaceship::ContractContract as spaceship;
use shippool::ContractContract as shippool;
use token_tia::ContractContract as token_tia;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    BOXMALL_WASM_BYTES => "../../res/boxmall.wasm",
    MOCKUSN_WASM_BYTES => "../../res/mock_usn.wasm",
    MAGICBOX_WASM_BYTES => "../../res/magicbox.wasm",
    SPACESHIP_WASM_BYTES => "../../res/spaceship.wasm",
    SHIPPOOL_WASM_BYTES => "../../res/shippool.wasm",
    TIA_WASM_BYTES => "../../res/token_tia.wasm",
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

fn deploy_contracts(
    root: &UserAccount,
    bank_id: AccountId,
    bank_u_id: AccountId,
    oracle_id: AccountId,
    risker_pool_id: AccountId,
    rank_pool_id: AccountId,
    luck_id: AccountId,
    owner_id: AccountId,
) -> (
    ContractAccount<boxmall>,
    ContractAccount<mock_usn>,
    ContractAccount<magicbox>,
    ContractAccount<spaceship>,
    ContractAccount<shippool>,
    ContractAccount<token_tia>,
) {
    let boxmall = deploy!(
        contract: boxmall,
        contract_id: "boxmall".to_string(),
        bytes: &BOXMALL_WASM_BYTES,
        signer_account: root,
        init_method: new(
            owner_id.clone(),
            "magicbox".parse().unwrap(),
            "mock_usn".parse().unwrap(),
            "token_tia".parse().unwrap(),
            bank_id.clone(),
            bank_u_id.clone(),
            oracle_id.clone(),
            risker_pool_id.clone(),
            rank_pool_id.clone(),
            "shippool".parse().unwrap(),
            luck_id.clone()
        )
    );

    let usn = deploy!(
        contract: mock_usn,
        contract_id: "mock_usn".to_string(),
        bytes: &MOCKUSN_WASM_BYTES,
        signer_account: root,
        init_method: new()
    );

    let magicbox = deploy!(
        contract: magicbox,
        contract_id: "magicbox".to_string(),
        bytes: &MAGICBOX_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(), "spaceship".parse().unwrap())
    );

    let spaceship = deploy!(
        contract: spaceship,
        contract_id: "spaceship".to_string(),
        bytes: &SPACESHIP_WASM_BYTES,
        signer_account: root,
        init_method: new(
            owner_id.clone(), 
            "magicbox".parse().unwrap(), 
            "shippool".parse().unwrap(),
            "shipmarket".parse().unwrap(),
            "auction".parse().unwrap(),
            "luckpool".parse().unwrap()
        )
    );

    let shippool = deploy!(
        contract: shippool,
        contract_id: "shippool".to_string(),
        bytes: &SHIPPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(), "spaceship".parse().unwrap(),"token_tia".parse().unwrap())
    );

    let token_tia = deploy!(
        contract: token_tia,
        contract_id: "token_tia".to_string(),
        bytes: &TIA_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(), "tia".to_string(), "tia".to_string(), 18)
    );
    
    (boxmall, usn, magicbox, spaceship, shippool, token_tia)
}

#[test]
fn sim_buys() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".parse().unwrap(), to_yocto("1000"));
    let user = root.create_user("user".parse().unwrap(), to_yocto("100"));

    let bank = root.create_user("bank".parse().unwrap(), to_yocto("100"));
    let bank_u = root.create_user("bank_u".parse().unwrap(), to_yocto("100"));
    let oracle = root.create_user("oracle".parse().unwrap(), to_yocto("100"));
    let risker_pool = root.create_user("risker_pool".parse().unwrap(), to_yocto("100"));
    let rank_pool = root.create_user("rank_pool".parse().unwrap(), to_yocto("100"));
    let luck = root.create_user("luck".parse().unwrap(), to_yocto("100"));

    let (boxmall, mock_usn, magicbox, spaceship, shippool, token_tia) = deploy_contracts(
        &root,
        bank.account_id(),
        bank_u.account_id(),
        oracle.account_id(),
        risker_pool.account_id(),
        rank_pool.account_id(),
        luck.account_id(),
        owner.account_id(),
    );
    println!("deploy_contracts(boxmall, mock_usn ... successfully)");

    // mint tia
    call!(
        owner,
        token_tia.mint(U128(100 * 10_u128.pow(18))),
        deposit = 1
    )
    .assert_success();

    // register user
    call!(
        user,
        token_tia.storage_deposit(None, None),
        deposit = to_yocto("0.00125")
    )
    .assert_success();

    // transfer 50 tia to user
    call!(
        owner,
        token_tia.ft_transfer(user.account_id(), U128(50 * 10_u128.pow(18)), None),
        deposit = 1
    )
    .assert_success();

    assert_eq!(
        view!(token_tia.ft_balance_of(user.account_id()))
            .unwrap_json::<U128>()
            .0,
        50 * 10_u128.pow(18)
    );
    assert_eq!(
        view!(token_tia.ft_balance_of(owner.account_id()))
            .unwrap_json::<U128>()
            .0,
        50 * 10_u128.pow(18)
    );
}
