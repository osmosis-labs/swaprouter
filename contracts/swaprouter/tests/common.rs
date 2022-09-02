use std::path::PathBuf;

use cosmwasm_std::Coin;
use osmosis_testing::{
    account::{Account, SigningAccount},
    app::App,
};
use swaprouter::msg::InstantiateMsg;

pub fn setup_test_env() -> (App, String, SigningAccount) {
    let app = App::new();

    // setup owner account
    let initial_balance = [
        Coin::new(1_000_000_000_000, "uosmo"),
        Coin::new(1_000_000_000_000, "uion"),
        Coin::new(1_000_000_000_000, "uatom"),
    ];
    let owner = app.init_account(&initial_balance);

    // create pools
    app.create_basic_pool(
        &owner.address(),
        &[Coin::new(1_000, "uion"), Coin::new(1_000, "uosmo")],
    );
    app.create_basic_pool(
        &owner.address(),
        &[Coin::new(1_000, "uatom"), Coin::new(1_000, "uosmo")],
    );
    app.create_basic_pool(
        &owner.address(),
        &[Coin::new(1_000, "uatom"), Coin::new(1_000, "uion")],
    );

    let code_id = app.store_code(&owner.address(), &get_wasm());

    let contract_address = app.instantiate_contract(
        &owner,
        code_id,
        &InstantiateMsg {
            owner: owner.address(),
        },
        &[],
        Some(&owner),
        None,
    );

    (app, contract_address, owner)
}

pub fn get_wasm() -> Vec<u8> {
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("swaprouter.wasm");
    std::fs::read(wasm_path).unwrap()
}
