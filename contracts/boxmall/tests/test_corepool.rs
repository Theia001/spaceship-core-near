use near_sdk::{
    env, near_bindgen, require, AccountId, PanicOnDefault, log
};
use near_sdk_sim::{
    init_simulator, view, call, deploy, to_yocto, ExecutionResult, ContractAccount, UserAccount, DEFAULT_GAS
};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;

use near_contract_standards::non_fungible_token::Token;

const DECIMAL: u32 = 24;

use boxmall::ContractContract as boxmall;
use mock_usn::ContractContract as mock_usn;
use magicbox::ContractContract as magicbox;
use spaceship::ContractContract as spaceship;
use shippool::ContractContract as shippool;
use token_tia::ContractContract as token_tia;
use riskerpool::ContractContract as riskerpool;
use rankpool::ContractContract as rankpool;
use trialpool::ContractContract as trialpool;
use corepool::ContractContract as corepool;


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    BOXMALL_WASM_BYTES => "../../res/boxmall.wasm",
    MOCKUSN_WASM_BYTES => "../../res/mock_usn.wasm",
    MAGICBOX_WASM_BYTES => "../../res/magicbox.wasm",
    SPACESHIP_WASM_BYTES => "../../res/spaceship.wasm",
    SHIPPOOL_WASM_BYTES => "../../res/shippool.wasm",
    TIA_WASM_BYTES => "../../res/token_tia.wasm",
    RISKERPOOL_WASM_BYTES => "../../res/riskerpool.wasm",
    RANKPOOL_WASM_BYTES => "../../res/rankpool.wasm",
    TRIALPOOL_WASM_BYTES => "../../res/trialpool.wasm",
    COREPOOL_WASM_BYTES => "../../res/corepool.wasm",
}


pub fn show_promises(r: &ExecutionResult) {
    for promise in r.promise_results() {
        println!("{:?}", promise);
    }
}

pub fn get_total_gas(r: &ExecutionResult) -> u64 {
    r.promise_results()
        .iter()
        .map(|x| x.as_ref().unwrap().gas_burnt().0)
        .sum()
}

pub fn get_logs(r: &ExecutionResult) -> Vec<String> {
    let mut logs: Vec<String> = vec![];
    r.promise_results().iter().map(
        |ex| {ex.as_ref().unwrap().logs().iter().map(
            |x| logs.push(x.clone())
        ).for_each(drop); logs.push("--------------------------".to_string())}
    ).for_each(drop);
    logs
}

pub fn get_error_count(r: &ExecutionResult) -> u32 {
    r.promise_errors().len() as u32
}

pub fn get_error_status(r: &ExecutionResult) -> String {
    format!("{:?}", r.promise_errors()[0].as_ref().unwrap().status())
}
/*
pub fn register_and_fund_mock_usn_account(contractacc: &ContractAccount<mock_usn>, account: &UserAccount, amount: u128){

    let out_come = near_sdk_sim::call!(
        *account,
        contractacc.mint(U128(amount * 10_u128.pow(DECIMAL))),
        deposit = 1
    );
    out_come.assert_success();
}
*/

pub fn register_and_fund_mock_usn_account( contractacc: &ContractAccount<mock_usn>, owner: &UserAccount, account: AccountId, amount: u128){
    // register account and deposit to account
    let mut out_come = near_sdk_sim::call!(
        *owner,
        contractacc.storage_deposit(Some(account.clone()),None),
        deposit = to_yocto("0.00125")
    );
    out_come.assert_success();
    //println!("{:#?}", out_come.promise_results());
    //println!("{:#?}", get_logs(&out_come));

    // transfer tia to account.
    out_come = near_sdk_sim::call!(
        *owner,
        contractacc.ft_transfer(account.clone(),U128(amount * 10_u128.pow(DECIMAL)),None),
        deposit = 1
    );
    out_come.assert_success();
    //println!("{:#?}", out_come.promise_results());
    //println!("{:#?}", get_logs(&out_come));
}

pub fn register_and_fund_tia_account( contractacc: &ContractAccount<token_tia>, owner: &UserAccount, account: AccountId, amount: u128){
    // register account and deposit to account
    let mut out_come = near_sdk_sim::call!(
        *owner,
        contractacc.storage_deposit(Some(account.clone()),None),
        deposit = to_yocto("0.00125")
    );
    out_come.assert_success();
    //println!("{:#?}", out_come.promise_results());
    //println!("{:#?}", get_logs(&out_come));

    // transfer tia to account.
    out_come = near_sdk_sim::call!(
        *owner,
        contractacc.ft_transfer(account.clone(),U128(amount * 10_u128.pow(DECIMAL)),None),
        deposit = 1
    );
    out_come.assert_success();
    //println!("{:#?}", out_come.promise_results());
    //println!("{:#?}", get_logs(&out_come));
}

fn deploy_contracts(
    root: &UserAccount,
    bank_id: AccountId,
    bank_u_id: AccountId,
    oracle_id: AccountId,
    risker_pool_id: AccountId,
    rank_pool_id: AccountId,
    luck_id: AccountId,
    owner_id: AccountId
) -> (ContractAccount<boxmall>, ContractAccount<mock_usn>,
      ContractAccount<magicbox>, 
      ContractAccount<spaceship>,
      ContractAccount<shippool>, ContractAccount<token_tia>,
      ContractAccount<riskerpool>, ContractAccount<rankpool>,
      ContractAccount<trialpool>, ContractAccount<corepool>
    ) {
    let boxmall_contract = deploy!(
        contract: boxmall,
        contract_id: "boxmall".to_string(),
        bytes: &BOXMALL_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(),
            "magicbox".parse().unwrap(),
            "mock_usn".parse().unwrap(),
            "token_tia".parse().unwrap(),
            bank_id.clone(),
            bank_u_id.clone(),
            oracle_id.clone(),
            "riskerpool".parse().unwrap(),
            "rankpool".parse().unwrap(),
            "shippool".parse().unwrap(),
            luck_id.clone()
        )
    );
    println!("deploy boxmall ... done");
    let usn_contract = deploy!(
        contract: mock_usn,
        contract_id: "mock_usn".to_string(),
        bytes: &MOCKUSN_WASM_BYTES,
        signer_account: root,
        init_method: new()
    );
    println!("deploy usn ... done");
    let magicbox_contract = deploy!(
        contract: magicbox,
        contract_id: "magicbox".to_string(),
        bytes: &MAGICBOX_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(),"spaceship".parse().unwrap())
    );
    println!("deploy magicbox ... done");

    let spaceship_contract = deploy!(
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
    println!("deploy spaceship ... done");

    let shippool_contract = deploy!(
        contract: shippool,
        contract_id: "shippool".to_string(),
        bytes: &SHIPPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(),"spaceship".parse().unwrap(),"token_tia".parse().unwrap())
    );
    println!("deploy shippool ... done");

    //
    let token_tia_contract = deploy!(
        contract: token_tia,
        contract_id: "token_tia".to_string(),
        bytes: &TIA_WASM_BYTES,
        signer_account: root,
        init_method: new(owner_id.clone(),"tia".to_string(),"tia".to_string(),18)
    );
    println!("deploy token_tia ... done");
    //
    //
    let riskerpool_contract = deploy!(
        contract: riskerpool,
        contract_id: "riskerpool".to_string(),
        bytes: &RISKERPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new(
            owner_id.clone(),
            "token_tia".parse().unwrap(),
            "boxmall".parse().unwrap()
        )
    );
    println!("deploy riskerpool ... done");
    //
    //
    let rankpool_contract = deploy!(
        contract: rankpool,
        contract_id: "rankpool".to_string(),
        bytes: &RANKPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new(
            owner_id.clone(),
            "token_tia".parse().unwrap(),
            "boxmall".parse().unwrap(),
            "shipmarket".parse().unwrap()
        )
    );
    println!("deploy rankpool ... done");

    //
    let trialpool_contract = deploy!(
        contract: trialpool,
        contract_id: "trialpool".to_string(),
        bytes: &TRIALPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new(
            owner_id.clone(),
            "token_tia".parse().unwrap(),
            "spaceship".parse().unwrap(),
            "boxmall".parse().unwrap(),
             1,
             U128(120_000_000_000_000_000),
             U128(to_yocto18("300")),
             U128(to_yocto18("600"))
        )
    );
    println!("deploy trialpool ... done");

    //
    let corepool_contract = deploy!(
        contract: corepool,
        contract_id: "corepool".to_string(),
        bytes: &COREPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new(
            owner_id.clone(),
            "token_tia".parse().unwrap(),
            "mock_usn".parse().unwrap(),
            "spaceship".parse().unwrap(),
            "boxmall".parse().unwrap(),
            1,
            U128(480_000_000_000_000_000),
            U128(to_yocto18("300")),
            U128(to_yocto18("600"))
        )
    );
    println!("deploy corepool ... done");

    //(boxmall_contract, usn_contract, magicbox_contract, spaceship_contract, shippool_contract, token_tia_contract)
    ( boxmall_contract, usn_contract, magicbox_contract, spaceship_contract, shippool_contract, 
      token_tia_contract, riskerpool_contract, rankpool_contract, trialpool_contract, corepool_contract )
}


pub fn to_yocto18(value: &str) -> u128 {
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(18);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(18 - power);
        part1 + part2
    } else {
        part1
    }
}

#[test]
fn test_corepool() {
    let root = init_simulator(None);
    let owner = root.create_user(AccountId::new_unchecked("owner".to_string()), to_yocto("1000"));
    let user = root.create_user(AccountId::new_unchecked("user".to_string()), to_yocto("1000"));
    let invitor = root.create_user(AccountId::new_unchecked("invitor".to_string()), to_yocto("1000"));


    let bank = root.create_user("bank".parse().unwrap(), to_yocto("100"));
    let bank_u = root.create_user("bank_u".parse().unwrap(), to_yocto("100"));
    let oracle = root.create_user("oracle".parse().unwrap(), to_yocto("100"));
    let risker_pool = root.create_user("risker_pool".parse().unwrap(), to_yocto("100"));
    let rank_pool = root.create_user("rank_pool".parse().unwrap(), to_yocto("100"));
    let luck = root.create_user("luck".parse().unwrap(), to_yocto("100"));
    let burned_id = root.create_user("00".parse().unwrap(), to_yocto("100"));

    let user_balance = user.account().unwrap().amount;
    println!("{:#?}",user_balance);


    let (boxmall, mock_usn,  magicbox, spaceship, shippool, 
        token_tia, riskerpool, rankpool, trialpool, corepool ) = deploy_contracts(
        &root, 
        bank.account_id(), 
        bank_u.account_id(), 
        oracle.account_id(),
        risker_pool.account_id(),
        rank_pool.account_id(),
        luck.account_id(),
        owner.account_id());
    println!("deploy_contracts(boxmall, mock_usn ... successfully)");
  
    //
    //
    let user_balance = user.account().unwrap().amount;
    println!("{:#?}",user_balance);


    //////////////////////////////////////////////////////////////////////////
    //
    // register and fund usn for users
    //
    //////////////////////////////////////////////////////////////////////////
    let out_come = call!(
        mock_usn.user_account, // only owner can mint
        mock_usn.mint(U128(1000000 * 10_u128.pow(DECIMAL))),
        deposit = 1
    );
    out_come.assert_success();
    //println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));

    // check storage_balance_bounds.  return to_yocto("0.00125")
    let storage_balance_bounds = view!(mock_usn.storage_balance_bounds());
    println!("mock_usn storage_balance_bounds: {:#?}", storage_balance_bounds.unwrap_json_value());

    //
    // mint usn for user
    register_and_fund_mock_usn_account(&mock_usn, &mock_usn.user_account, user.account_id().clone(),2000);
    let balance = view!(mock_usn.ft_balance_of(user.account_id()));
    println!("user: mock_usn.ft_balance_of: {:#?}", balance.unwrap_json_value());

    // mint usn for boxmall
    register_and_fund_mock_usn_account(&mock_usn, &mock_usn.user_account, boxmall.user_account.account_id(),2000);
    let balance = view!(mock_usn.ft_balance_of(boxmall.user_account.account_id()));
    println!("boxmall.user_account: mock_usn.ft_balance_of: {:#?}", balance.unwrap_json_value());

    // mint usn for bankU
    register_and_fund_mock_usn_account(&mock_usn, &mock_usn.user_account, bank_u.account_id(),2000);
    let balance = view!(mock_usn.ft_balance_of(bank_u.account_id()));
    println!("bank_u.user_account: mock_usn.ft_balance_of: {:#?}", balance.unwrap_json_value());

    // mint usn for invitor
    register_and_fund_mock_usn_account(&mock_usn, &mock_usn.user_account, invitor.account_id(),2000);
    let balance = view!(mock_usn.ft_balance_of(invitor.account_id()));
    println!("invitor.user_account: mock_usn.ft_balance_of: {:#?}", balance.unwrap_json_value());

    // mint usn for invitor
    register_and_fund_mock_usn_account(&mock_usn, &mock_usn.user_account, corepool.user_account.account_id(),2000);
    let balance = view!(mock_usn.ft_balance_of(corepool.user_account.account_id()));
    println!("corepool.user_account: mock_usn.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //////////////////////////////////////////////////////////////////////////
    //
    // register and fund token_tia for users
    //
    //////////////////////////////////////////////////////////////////////////
    let mut out_come = call!(
        owner, // only owner can mint
        token_tia.mint(U128(1000000000 * 10_u128.pow(DECIMAL))),
        deposit = 1
    );
    out_come.assert_success();
    //println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));

    // check storage_balance_bounds.  return to_yocto("0.00125")
    let storage_balance_bounds = view!(token_tia.storage_balance_bounds());
    println!("token_tia storage_balance_bounds: {:#?}", storage_balance_bounds.unwrap_json_value());

    ///////////////////////////////////////////////////////////
    // register and fund 'boxmall' account
    let balance = view!(token_tia.ft_balance_of(boxmall.user_account.account_id()));
    println!("boxmall: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    register_and_fund_tia_account(&token_tia, &owner, boxmall.user_account.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(boxmall.user_account.account_id()));
    println!("boxmall: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'user' account
    register_and_fund_tia_account(&token_tia, &owner, user.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(user.account_id()));
    println!("user: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'invitor' account
    register_and_fund_tia_account(&token_tia, &owner, invitor.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(invitor.account_id()));
    println!("invitor: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'shippool' account
    register_and_fund_tia_account(&token_tia, &owner, shippool.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(shippool.account_id()));
    println!("shippool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'bank' account
    register_and_fund_tia_account(&token_tia, &owner, bank.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(bank.account_id()));
    println!("bank: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'luck' account
    register_and_fund_tia_account(&token_tia, &owner, luck.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(luck.account_id()));
    println!("luck: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'riskerpool' account
    register_and_fund_tia_account(&token_tia, &owner, riskerpool.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(riskerpool.account_id()));
    println!("riskerpool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'rankpool' account
    register_and_fund_tia_account(&token_tia, &owner, rankpool.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(rankpool.account_id()));
    println!("rankpool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'trialpool' account with 2975908
    register_and_fund_tia_account(&token_tia, &owner, trialpool.account_id().clone(), 2975908);

    let balance = view!(token_tia.ft_balance_of(trialpool.account_id()));
    println!("trialpool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'burned' account
    register_and_fund_tia_account(&token_tia, &owner, burned_id.account_id().clone(), 2000);

    let balance = view!(token_tia.ft_balance_of(burned_id.account_id()));
    println!("burned_id: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    //////////////////////////////////////////////
    // register and fund 'corepool' account with 11903632
    register_and_fund_tia_account(&token_tia, &owner, corepool.account_id().clone(), 11903632);

    let balance = view!(token_tia.ft_balance_of(corepool.account_id()));
    println!("corepool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());
    //

    /////////////////////////////////////////////////////////////////////////////////////////////////////////
    // 1. test mock_usn.ft_transfer
    println!("------------------------------ 1 ------------------------------");
    out_come = call!(
        user,
        mock_usn.ft_transfer(boxmall.user_account.account_id().clone(), to_yocto18("0.01").into(), None),
        deposit = 1
    );
    out_come.assert_success();

    // 2. test token_tia.ft_transfer
    println!("------------------------------ 2 ------------------------------");
    out_come = call!(
        user,
        token_tia.ft_transfer(boxmall.user_account.account_id().clone(), to_yocto18("0.01").into(), None),
        deposit = 1
    );
    out_come.assert_success();


    // 3. test boxmall.set_num_limit
    println!("------------------------------ 3 ------------------------------");
    out_come = call!(
        owner,
        boxmall.set_num_limit(10),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());

    out_come = call!(
        owner,
        magicbox.set_num_limit(10),
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());

    // 4. test boxmall.add_ubox_sale
    println!("------------------------------ 4 ------------------------------");
    out_come = call!(
        owner,
        boxmall.add_ubox_sale(1655208000,1655218749,100,U128(5)),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    

    // 5. test buy ubox for boxmall.ft_on_transfer on usn token
    // set_switch
    println!("------------------------------ 5.1 ------------------------------");

    out_come = call!(
        owner,
        boxmall.set_switch(true,true),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));

    
    // note: for ubox, no deduct
    println!("------------------------------ 5.2 ------------------------------");
    let msg: String = String::from("{\"box_type\": \"buy_u\",\"num\": 2}");

    out_come = call!(
        user,
        mock_usn.ft_transfer_call(boxmall.user_account.account_id(), to_yocto18("10").into(), None, msg),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));

    // 6. test buy sbox for boxmall.ft_on_transfer on token_tia token
    // 6.1 set invite and bind a invitor
    println!("------------------------------ 6.1 ------------------------------");
    //
    out_come = call!(
        user,
        boxmall.bind(invitor.account_id()),
        //boxmall.bind(user.account_id()), // should be error
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    //

    /* 
    println!("------------------------------ 6.1.1 ------------------------------");
    out_come = call!(
        user,
        boxmall.bind(invitor.account_id()), // bind again, should be error
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    */

    // 6.2 getsbox price
    println!("------------------------------ 6.2 ------------------------------");
    out_come = call!(
        owner,
        boxmall.get_sbox_price(),
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());


    // 6.3 setsbox price
    println!("------------------------------ 6.3 ------------------------------");
    out_come = call!(
        owner,
        boxmall.set_sbox_price(U128(to_yocto18("10"))),
        deposit = 1
        //gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());

    // 6.4 getsbox price again to double check
    println!("------------------------------ 6.4 ------------------------------");
    out_come = call!(
        owner,
        boxmall.get_sbox_price(),
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());

    // 6.5 add shipwallet balance
    println!("------------------------------ 6.5 ------------------------------");
    out_come = call!(
        owner,
        boxmall.ship_wallet_add(user.account_id(),U128(to_yocto18("30"))),
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
 
    // 6.6 get shipwallet balance
    //
    println!("------------------------------ 6.6 ------------------------------");
    out_come = call!(
        owner,
        boxmall.ship_wallet_balance_of(user.account_id()),
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());


    // 6.7
    println!("------------------------------ 6.7 ------------------------------");
    /*/
    let msg: String = String::from("{
        \"box_type\": \"buy_s\",
        \"num\": 10,
    }");
    */
    let msg: String = String::from("{\"box_type\": \"buy_s\",\"num\": 10}");

    out_come = call!(
        user,
        token_tia.ft_transfer_call(boxmall.user_account.account_id(), to_yocto18("70").into(), None, msg),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    //println!("{:#?}",out_come.profile_data());

    // test openbox
    // 7.1 get_box_list()
    println!("------------------------------ 7.1 ------------------------------");
    out_come = call!(
        user,
        magicbox.get_box_list(user.account_id(), 0, 100),
        deposit = 1
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    // 7.2 openbox
    println!("------------------------------ 7.2 ------------------------------");
    //let box_token_ids: Vec<String> = vec!["0:1".to_string(),"1:1".to_string(),"2:2".to_string(),"3:2".to_string()];
    let box_token_ids: Vec<String> = vec!["0:2".to_string(),"1:2".to_string(),"2:2".to_string(),"3:2".to_string(),"4:2".to_string(),"5:2".to_string(),"6:2".to_string(),
    "7:2".to_string(),"8:2".to_string(),"9:2".to_string()];
    out_come = call!(
        user,
        magicbox.open_box(box_token_ids),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    
    //
    // 7.3 get shipwallet balance to check balance
    //
    println!("------------------------------ 7.3 ------------------------------");
    out_come = call!(
        owner,
        boxmall.ship_wallet_balance_of(user.account_id()),
        //deposit = 1
        gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    
    //
    // 7.4 get balance of misc account 
    //
    println!("------------------------------ 7.4 ------------------------------");
    let balance = view!(token_tia.ft_balance_of(luck.account_id()));
    println!("luck: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    let balance = view!(token_tia.ft_balance_of(bank.account_id()));
    println!("bank: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    let balance = view!(token_tia.ft_balance_of(shippool.account_id()));
    println!("shippool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    let balance = view!(token_tia.ft_balance_of(riskerpool.account_id()));
    println!("riskerpool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    let balance = view!(token_tia.ft_balance_of(rankpool.account_id()));
    println!("rankpool: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    let balance = view!(token_tia.ft_balance_of(invitor.account_id()));
    println!("invitor: token_tia.ft_balance_of: {:#?}", balance.unwrap_json_value());

    //
    // 8 testing corepool 
    // 8.0 buy slot by token_tia.ft_transfer_call
    // 8.1 stake amout by mock_usn.ft_transfer_call
    // 8.2 get nft by spaceship.nft_tokens_for_owner
    // 8.3 stake nft by spaceship.nft_transfer_call
    // 8.4 get_slot_list 
    println!("------------------------------ 8.0 buy slot(TIA)---------------------------");
    let msg: String = String::from("2");
    out_come = call!(
        user,
        token_tia.ft_transfer_call(corepool.user_account.account_id(), to_yocto18("300").into(), None, msg),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));

    println!("------------------------------ 8.1 stake amount(USN)---------------------------");
    out_come = call!(
        user,
        mock_usn.ft_transfer_call(corepool.user_account.account_id(), to_yocto18("20").into(), None, "".to_string()),
        deposit = 1
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));

    println!("------------------------------ 8.2 get nft");
    let msg: String = String::from("50");

    out_come = call!(
        user,
        spaceship.nft_tokens_for_owner(
            user.account_id(), 
            Some(U128(0)), 
            Some(100)
            ),
        deposit = 1
        //gas = near_sdk_sim::DEFAULT_GAS
    );
    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));   

    println!("------------------------------ 8.3 stake nft to slot(2)");
    //
    let msg: String = String::from("2");
    out_come = call!(
        user,
        spaceship.nft_transfer_call(
            corepool.user_account.account_id(), 
            //"1:4:1:11".to_string(), 
            "1:1:30:11".to_string(), 
            None,
            None,
            msg),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));

    println!("------------------------------ 8.4 get_slot_list");
    //   
    out_come = call!(
        user,
        corepool.get_slot_list(user.account_id()),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));

    println!("------------------------------ 8.5 claim");
    //   
    out_come = call!(
        user,
        corepool.claim(),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));

    println!("------------------------------ 8.6 withdraw slot(2)");
    //   
    out_come = call!(
        user,
        corepool.withdraw(2),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));


    println!("------------------------------ 8.7 stake nft to slot(0)");
    //
    let msg: String = String::from("0");
    out_come = call!(
        user,
        spaceship.nft_transfer_call(
            corepool.user_account.account_id(), 
            //"1:4:1:11".to_string(), 
            "1:1:30:11".to_string(), 
            None,
            None,
            msg),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));


    println!("------------------------------ 8.8 claim");
    //   
    out_come = call!(
        user,
        corepool.claim(),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));

    println!("------------------------------ 8.9 withdraw slot(0)");
    //   
    out_come = call!(
        user,
        corepool.withdraw(0),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));


    println!("------------------------------ 8.10 get_slot_list");
    //   
    out_come = call!(
        user,
        corepool.get_slot_list(user.account_id()),
        1,
        near_sdk_sim::DEFAULT_GAS
    );

    out_come.assert_success();
    println!("{:#?}", out_come.promise_results());
    println!("{:#?}", get_logs(&out_come));
    println!("transfer_call cost {} Tgas", get_total_gas(&out_come) / 10_u64.pow(12));


}
