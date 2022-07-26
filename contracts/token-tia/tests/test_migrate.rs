mod setup;
use crate::setup::*;

#[test]
fn test_update(){
    let e = Env::init_with_contract(previous_wasm_bytes());
    let users = Users::init(&e);

    assert_err!(
        e.upgrade_contract(&users.alice, cur_wasm_bytes()),
        "ERR_NOT_ALLOWED"
    );

    e.upgrade_contract(&e.owner, cur_wasm_bytes()).assert_success();
    assert_eq!(e.get_metadata().version, "0.0.1".to_string());
}