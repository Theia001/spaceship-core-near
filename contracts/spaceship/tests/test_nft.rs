mod setup;
use crate::setup::*;

#[test]
fn locate_overflow_bug() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let alice = e
        .root
        .create_user("alice".parse().unwrap(), to_yocto("100"));

    e.batch_mint(
        &e.magicbox,
        alice.account_id(),
        vec!["4".to_string(), "4".to_string()],
        vec!["1".to_string(), "2".to_string()],
    )
    .assert_success();

    let ships = e.get_spaceship_list_for_owner(alice.account_id(), None, None);
    let token1_id = ships.get(0).unwrap().to_token_id();

    // 1. user sell nft
    let outcome = e.nft_transfer_call(
        &alice,
        e.shipmarket.account_id(),
        token1_id.clone(),
        "pass".to_string(),
    );
    outcome.assert_success();

    // 2. user cancel sell nft
    let outcome = e.nft_transfer(
        &e.shipmarket.user_account,
        alice.account_id(),
        token1_id.clone(),
    );
    outcome.assert_success();

    // 3. user sell nft with error id
    assert_err!(
        e.nft_transfer_call(
            &alice,
            e.shipmarket.account_id(),
            "1:3:1:25".to_string(),
            "return".to_string(),
        ),
        "Token not found"
    );

    // show_promises(&outcome);
}

#[test]
fn batch_mint() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    assert_err!(
        e.batch_mint(
            &user,
            user.account_id(),
            vec!["1".to_string()],
            vec!["3".to_string()],
        ),
        "ERR_NOT_ALLOWED"
    );

    // fill all sub type of D class
    let outcome = e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec![
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
        ],
        vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
            "9".to_string(),
            "10".to_string(),
            "11".to_string(),
            "12".to_string(),
            "13".to_string(),
            "14".to_string(),
            "15".to_string(),
            "16".to_string(),
        ],
    );
    outcome.assert_success();
    println!(
        "batch mint D[1-16] spaceship, costs {} Tgas",
        total_consumed_gas(&outcome) / 10_u64.pow(12)
    );
    let outcome = e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec![
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
        ],
        vec![
            "17".to_string(),
            "18".to_string(),
            "19".to_string(),
            "20".to_string(),
            "21".to_string(),
            "22".to_string(),
            "23".to_string(),
            "24".to_string(),
            "25".to_string(),
            "26".to_string(),
            "27".to_string(),
            "28".to_string(),
            "29".to_string(),
            "30".to_string(),
            "31".to_string(),
            "32".to_string(),
        ],
    );
    outcome.assert_success();
    println!(
        "batch mint D[17-32] spaceship, costs {} Tgas",
        total_consumed_gas(&outcome) / 10_u64.pow(12)
    );
    assert_eq!(e.get_spaceship_supply().supply, U128(36));

    // fill all sub type of C class
    let outcome = e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec![
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
            "2".to_string(),
        ],
        vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
            "9".to_string(),
            "10".to_string(),
            "11".to_string(),
            "12".to_string(),
            "13".to_string(),
            "14".to_string(),
            "15".to_string(),
            "16".to_string(),
        ],
    );
    outcome.assert_success();
    println!(
        "batch mint C[1-16] spaceship, costs {} Tgas",
        total_consumed_gas(&outcome) / 10_u64.pow(12)
    );
    assert_eq!(e.get_spaceship_supply().supply, U128(52));
    // fill all sub type of B class
    let outcome = e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec![
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
            "3".to_string(),
        ],
        vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
        ],
    );
    outcome.assert_success();
    println!(
        "batch mint B[1-8] spaceship, costs {} Tgas",
        total_consumed_gas(&outcome) / 10_u64.pow(12)
    );
    assert_eq!(e.get_spaceship_supply().supply, U128(60));

    // fill all sub type of A class
    let outcome = e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec![
            "4".to_string(),
            "4".to_string(),
            "4".to_string(),
            "4".to_string(),
        ],
        vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
        ],
    );
    outcome.assert_success();
    println!(
        "batch mint A spaceship, costs {} Tgas",
        total_consumed_gas(&outcome) / 10_u64.pow(12)
    );
    assert_eq!(e.get_spaceship_supply().supply, U128(64));

    for i in 0..10 {
        let outcome = e.batch_mint(
            &e.magicbox,
            user.account_id(),
            vec![
                "1".to_string(),
                "1".to_string(),
                "1".to_string(),
                "1".to_string(),
                "1".to_string(),
                "1".to_string(),
                "2".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ],
            vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "1".to_string(),
                "2".to_string(),
                "1".to_string(),
                "1".to_string(),
            ],
        );
        outcome.assert_success();
        // show_promises(&outcome);
        println!(
            "step {}: batch mint 10 spaceship, costs {} Tgas",
            i,
            total_consumed_gas(&outcome) / 10_u64.pow(12)
        );
    }
}

#[test]
fn transfer_spaceship() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let alice = e
        .root
        .create_user("alice".parse().unwrap(), to_yocto("100"));
    let bob = e.root.create_user("bob".parse().unwrap(), to_yocto("100"));

    e.batch_mint(
        &e.magicbox,
        alice.account_id(),
        vec!["4".to_string(), "4".to_string()],
        vec!["1".to_string(), "2".to_string()],
    )
    .assert_success();
    assert_eq!(e.get_balance_type_of(alice.account_id(), 4), 2);
    assert_eq!(e.get_balance_subtype_of(alice.account_id(), 4, 1), 1);
    assert_eq!(e.get_balance_subtype_of(alice.account_id(), 4, 2), 1);

    let ships = e.get_spaceship_list_for_owner(alice.account_id(), None, None);
    let token1_id = ships.get(0).unwrap().to_token_id();
    let token2_id = ships.get(1).unwrap().to_token_id();

    let outcome = e.nft_transfer(&alice, bob.account_id(), token1_id.clone());
    outcome.assert_success();
    assert_eq!(e.get_balance_type_of(alice.account_id(), 4), 1);
    assert_eq!(e.get_balance_subtype_of(alice.account_id(), 4, 2), 1);
    assert_eq!(e.get_balance_type_of(bob.account_id(), 4), 1);
    assert_eq!(e.get_balance_subtype_of(bob.account_id(), 4, 1), 1);

    let outcome = e.nft_transfer_call(
        &alice,
        e.shipmarket.account_id(),
        token2_id.clone(),
        "pass".to_string(),
    );
    outcome.assert_success();
    assert_eq!(e.get_balance_type_of(alice.account_id(), 4), 0);
    assert_eq!(e.get_balance_subtype_of(alice.account_id(), 4, 2), 0);
    assert_eq!(e.get_balance_type_of(bob.account_id(), 4), 1);
    assert_eq!(e.get_balance_subtype_of(bob.account_id(), 4, 1), 1);
    assert_eq!(e.get_balance_type_of(e.shipmarket.account_id(), 4), 1);
    assert_eq!(e.get_balance_subtype_of(e.shipmarket.account_id(), 4, 2), 1);

    let outcome = e.nft_transfer_call(
        &bob,
        e.shipmarket.account_id(),
        token1_id.clone(),
        "return".to_string(),
    );
    outcome.assert_success();
    assert_eq!(e.get_balance_type_of(alice.account_id(), 4), 0);
    assert_eq!(e.get_balance_subtype_of(alice.account_id(), 4, 2), 0);
    assert_eq!(e.get_balance_type_of(bob.account_id(), 4), 1);
    assert_eq!(e.get_balance_subtype_of(bob.account_id(), 4, 1), 1);
    assert_eq!(e.get_balance_type_of(e.shipmarket.account_id(), 4), 1);
    assert_eq!(e.get_balance_subtype_of(e.shipmarket.account_id(), 4, 2), 1);
    // show_promises(&outcome);
}

#[test]
fn nft_payout() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    let outcome = e.nft_payout(&e.auction, user.account_id(), String::from("1:5:1:240"));
    outcome.assert_success();

    let ships = e.get_spaceship_list_for_owner(user.account_id(), None, None);
    assert_eq!(ships.len(), 1);
    assert_eq!(
        ships.get(0).unwrap().to_token_id(),
        String::from("1:5:1:240")
    );
}

#[test]
fn user_burn() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec!["4".to_string(), "4".to_string()],
        vec!["1".to_string(), "2".to_string()],
    )
    .assert_success();

    let ships = e.get_spaceship_list_for_owner(user.account_id(), None, None);
    let token1_id = ships.get(0).unwrap().to_token_id();
    let token2_id = ships.get(1).unwrap().to_token_id();

    let outcome = e.user_burn(&user, token1_id.clone());
    outcome.assert_success();

    let ships = e.get_spaceship_list_for_owner(user.account_id(), None, None);
    assert_eq!(ships.len(), 1);
    assert_eq!(ships.get(0).unwrap().to_token_id(), token2_id);
}

#[test]
fn upgrade_spaceship() {
    let e = Env::init_with_contract(spaceship_wasm_bytes());
    let user = e.root.create_user("user".parse().unwrap(), to_yocto("100"));

    e.mint_eng(&e.owner, U128(100)).assert_success();
    e.eng_ft_register(&user, None).assert_success();
    e.eng_ft_transfer(&&e.owner, user.account_id(), U128(50), None)
        .assert_success();
    assert_eq!(e.get_eng_balance_of(user.account_id()), U128(50));

    e.batch_mint(
        &e.magicbox,
        user.account_id(),
        vec!["1".to_string(), "1".to_string()],
        vec!["1".to_string(), "2".to_string()],
    )
    .assert_success();
    assert_eq!(e.get_balance_type_of(user.account_id(), 1), 2);
    assert_eq!(e.get_balance_subtype_of(user.account_id(), 1, 1), 1);
    assert_eq!(e.get_balance_subtype_of(user.account_id(), 1, 2), 1);

    let ships = e.get_spaceship_list_for_owner(user.account_id(), None, None);
    let token1_id = ships.get(0).unwrap().to_token_id();
    let token2_id = ships.get(1).unwrap().to_token_id();

    let outcome = e.upgrade_spaceship(
        &&e.shipmarket.user_account,
        user.account_id(),
        token1_id,
        token2_id,
        1,
        U128(50),
    );
    outcome.assert_success();
    assert_eq!(e.get_balance_type_of(user.account_id(), 1), 0);
    assert_eq!(e.get_balance_type_of(user.account_id(), 2), 1);
    assert_eq!(e.get_balance_subtype_of(user.account_id(), 2, 1), 1);

    assert_eq!(e.get_eng_balance_of(user.account_id()), U128(0));

    let ships = e.get_spaceship_list_for_owner(user.account_id(), None, None);
    assert_eq!(ships.len(), 1);

    let supplies = e.get_spaceship_supply();
    assert_eq!(supplies.burned, U128(2));
    assert_eq!(supplies.supply, U128(5));
    assert_eq!(supplies.owners, U128(2));
}
