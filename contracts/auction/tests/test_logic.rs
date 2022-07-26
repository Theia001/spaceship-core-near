mod setup;
use auction::{AuctionInfo, DEFAULT_DURATION_SEC, MAX_DURATION_SEC};

use crate::setup::*;

#[test]
fn bid() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    e.mint_nft("example_token".to_string()).assert_success();
    assert_eq!(e.nft_owner("example_token".to_string()), e.owner.account_id());

    e.add_auction_info(&e.owner, "example_token".to_string(), 100_u128, 10_u128, 1_000, 1000 + DEFAULT_DURATION_SEC).assert_success();

    e.mint_ft(&user, 1000).assert_success();
    assert_err!(e.ft_bid(&user, "0".to_string(), 100), "invalid token contract id");

    e.mint_usn(&user, 1000).assert_success();
    assert_err!(e.usn_bid(&user, "abc".to_string(), 100), "msg must contain all digits");
    assert_err!(e.usn_bid(&user, "1".to_string(), 100), "invalid auction_id");
    assert_err!(e.usn_bid(&user, "0".to_string(), 100), "ERR_AUCTION_NOT_START");

    // passes start time of the auction
    e.set_time(1_100);
    assert_err!(e.usn_bid(&user, "0".to_string(), 110), "ERR_INVALID_PRICE");
    
    e.usn_bid(&user, "0".to_string(), 100).assert_success();
    assert_eq!(e.get_auction_by_id(0).unwrap(), AuctionInfo {
        buyer: user.account_id(),
        token_id: "example_token".to_string(),
        price: 100_u128,
        up_price: 10_u128,
        start: 1_000,
        end: 1_000 + DEFAULT_DURATION_SEC + DEFAULT_DURATION_SEC,
        team_fund: 100,
        claimed: false,
        team_fund_claimed: false,
    });
    assert_eq!(e.usn_balance(&user), 1000 - 100);

    // time goes by
    e.set_time(1500);
    let alice = e.root.create_user("alice".parse().unwrap(), to_yocto("100"));
    e.mint_usn(&alice, 1000).assert_success();
    e.usn_bid(&alice, "0".to_string(), 110).assert_success();
    assert_eq!(e.get_auction_by_id(0).unwrap(), AuctionInfo {
        buyer: alice.account_id(),
        token_id: "example_token".to_string(),
        price: 110_u128,
        up_price: 10_u128,
        start: 1_000,
        end: 1_000 + 3 * DEFAULT_DURATION_SEC,
        team_fund: 109,
        claimed: false,
        team_fund_claimed: false,
    });
    assert_eq!(e.usn_balance(&user), 1000 - 100 + 101);
    assert_eq!(e.usn_balance(&alice), 1000 - 110);

    assert_err!(e.claim(&user, 1), "invalid auction_id");
    assert_err!(e.claim(&user, 0), "ERR_AUCTION_STILL_RUNNING");

    // passes end time of the auction 
    e.set_time((1_000 + 3 * DEFAULT_DURATION_SEC + 1) as u32);
    assert_err!(e.usn_bid(&user, "0".to_string(), 120), "ERR_AUCTION_ENDED");
    assert_err!(e.claim(&user, 0), "ERR_NOT_AUCTION_WINNER");

    e.claim(&alice, 0).assert_success();
    assert_eq!(e.nft_owner("example_token".to_string()), alice.account_id());
    assert_eq!(e.usn_balance(&e.team), 109);

    assert_err!(e.claim(&alice, 0), "ERR_AUCTION_ALREADY_CLAIMED");
}

#[test]
fn extend_end_of_auction() {
    let e = Env::init_with_contract(auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));
    e.add_auction_info(&e.owner, "token".to_string(), 100, 10, 1_000, 1000 + MAX_DURATION_SEC).assert_success();
    e.mint_usn(&user, 1000).assert_success();

    e.set_time(1010);
    e.usn_bid(&user, "0".to_string(), 100).assert_success();
    assert_eq!(e.get_auction_by_id(0).unwrap().end, 1010 + MAX_DURATION_SEC);

    e.set_time((1000 + MAX_DURATION_SEC) as u32);
    e.usn_bid(&user, "0".to_string(), 110).assert_success();
    assert_eq!(e.get_auction_by_id(0).unwrap().end, 1010 + MAX_DURATION_SEC + DEFAULT_DURATION_SEC);
}