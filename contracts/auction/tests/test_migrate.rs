mod setup;
use crate::setup::*;

#[test]
fn update() {
    let e = Env::init_with_contract(previous_auction_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_err!(e.upgrade_contract(&user, auction_wasm_bytes()), "ERR_NOT_ALLOWED");

    e.upgrade_contract(&e.owner, auction_wasm_bytes()).assert_success();
    assert_eq!(e.get_metadata().version, "0.0.2".to_string());
}
