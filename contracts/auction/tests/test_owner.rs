mod setup;
use auction::{AuctionId, AuctionInfo, DEFAULT_DURATION_SEC, ONE_DAY_IN_SECS};

use crate::setup::*;

#[test]
fn change_owner() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_eq!(e.get_metadata().owner_id, e.owner.account_id());
    
    // error scene 
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(e.set_owner(&user, &e.owner, 0), "Requires attached deposit of exactly 1 yoctoNEAR");

    // 2: ERR_NOT_ALLOWED
    assert_err!(e.set_owner(&user, &e.owner, 1), "ERR_NOT_ALLOWED");

    // success
    e.set_owner(&e.owner, &user, 1).assert_success();
    assert_eq!(e.get_metadata().owner_id, user.account_id());
}

#[test]
fn manage_auction() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_err!(e.add_auction_info(&user, "token".to_string(), 100, 10, 100, 200), "ERR_NOT_ALLOWED");
    assert_err!(e.add_auction_info(&e.owner, "token".to_string(), 100, 10, 100, 50), "ERR_INVALID_END_TIME");
    assert_err!(e.add_auction_info(&e.owner, "token".to_string(), 100, 10, 50, 100), "ERR_INVALID_START_TIME");

    let outcome = e.add_auction_info(&e.owner, "token".to_string(), 100_u128, 10_u128, 150, 200);
    outcome.assert_success();
    assert_eq!(outcome.unwrap_json::<AuctionId>(), 0);

    assert_eq!(e.get_auction_by_id(0).unwrap(), AuctionInfo {
        buyer: e.owner.account_id(),
        token_id: "token".to_string(),
        price: 100_u128,
        up_price: 10_u128,
        start: 150,
        end: 200,
        team_fund: 0,
        claimed: false,
        team_fund_claimed: false,
    });
    assert_eq!(e.get_auction_count(), 1_u64);

    assert_err!(e.update_auction_info(&user, 0, 100, 10, 100, 200), "ERR_NOT_ALLOWED");
    assert_err!(e.update_auction_info(&e.owner, 1, 100, 10, 100, 50), "Invalid auction_id");
    assert_err!(e.update_auction_info(&e.owner, 0, 100, 10, 100, 50), "ERR_INVALID_END_TIME");
    assert_err!(e.update_auction_info(&e.owner, 0, 100, 10, 50, 100), "ERR_INVALID_START_TIME");

    e.update_auction_info(&e.owner, 0, 110, 20, 180, 250).assert_success();

    assert_eq!(e.get_auction_by_id(0).unwrap(), AuctionInfo {
        buyer: e.owner.account_id(),
        token_id: "token".to_string(),
        price: 110_u128,
        up_price: 20_u128,
        start: 180,
        end: 250,
        team_fund: 0,
        claimed: false,
        team_fund_claimed: false,
    });
    assert_eq!(e.get_auction_count(), 1_u64);
}

#[test]
fn manage_team() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_err!(e.set_team_account(&user, user.account_id()), "ERR_NOT_ALLOWED");
    assert_eq!(e.get_metadata().team_id, e.team.account_id());

    e.set_team_account(&e.owner, user.account_id()).assert_success();
    assert_eq!(e.get_metadata().team_id, user.account_id());
}

#[test]
fn manage_duration() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_err!(e.set_extend_duration_sec(&user, 7200), "ERR_NOT_ALLOWED");
    assert_eq!(e.get_metadata().duration_sec, DEFAULT_DURATION_SEC);

    e.set_extend_duration_sec(&e.owner, 7200).assert_success();
    assert_eq!(e.get_metadata().duration_sec, 7200);
}

#[test]
fn team_withdraw() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    e.add_auction_info(&e.owner, "token".to_string(), 100_u128, 10_u128, 1_000, 1000 + DEFAULT_DURATION_SEC).assert_success();

    e.mint_usn(&user, 1000).assert_success();
    e.skip_time(1_000);
    e.usn_bid(&user, "0".to_string(), 100).assert_success();

    assert_err!(e.team_withdraw(&user, 0), "ERR_NOT_ALLOWED");
    assert_err!(e.team_withdraw(&e.owner, 1), "Invalid auction_id");
    assert_err!(e.team_withdraw(&e.owner, 0), "Auction: must be 24 hours after owner not claimed");

    e.skip_time((2*DEFAULT_DURATION_SEC + ONE_DAY_IN_SECS) as u32);
    e.team_withdraw(&e.owner, 0).assert_success();
    assert_eq!(e.usn_balance(&e.team), 100);
    assert_err!(e.team_withdraw(&e.owner, 0), "Auction: already claimed");
}