mod setup;
use crate::setup::*;

#[test]
fn change_owner() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_eq!(e.get_metadata().owner_id, e.owner.account_id());

    // error scene
    // 1 : Requires attached deposit of exactly 1 yoctoNEAR
    assert_err!(
        e.set_owner(&user, &e.owner, 0),
        "Requires attached deposit of exactly 1 yoctoNEAR"
    );

    // 2: ERR_NOT_ALLOWED
    assert_err!(e.set_owner(&user, &e.owner, 1), "ERR_NOT_ALLOWED");

    // success
    e.set_owner(&e.owner, &user, 1).assert_success();
    assert_eq!(e.get_metadata().owner_id, user.account_id());
}

#[test]
fn mint_eng() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    // success
    e.mint_eng(&e.owner, U128(100)).assert_success();
    assert_eq!(e.get_eng_balance_of(e.owner.account_id()), U128(100));
}

#[test]
fn set_eng_icon() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    // success
    e.set_eng_icon(&e.owner, String::from("abc"))
        .assert_success();
    assert_eq!(e.get_eng_metadata().icon, Some(String::from("abc")));
}

#[test]
fn set_ship_icon() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    // success
    e.set_ship_icon(&e.owner, String::from("1:2"), String::from("abc"))
        .assert_success();
    assert_eq!(
        e.get_ship_icon(String::from("1:2")),
        Some(String::from("abc"))
    );
    assert_eq!(e.get_ship_icon(String::from("2:2")), None);
}
