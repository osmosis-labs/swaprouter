use std::path::PathBuf;

use cosmwasm_std::Coin;
use osmosis_testing::account::{Account, SigningAccount};
use osmosis_testing::runner::app::App;
use osmosis_testing::x::gamm::Gamm;
use osmosis_testing::x::wasm::Wasm;
use osmosis_testing::x::AsModule;
use swaprouter::msg::InstantiateMsg;

pub struct TestEnv {
    pub app: App,
    pub contract_address: String,
    pub owner: SigningAccount,
}
impl TestEnv {
    pub fn new() -> Self {
        let app = App::new();
        let gamm = app.as_module::<Gamm<_>>();
        let wasm = app.as_module::<Wasm<_>>();

        // setup owner account
        let initial_balance = [
            Coin::new(1_000_000_000_000, "uosmo"),
            Coin::new(1_000_000_000_000, "uion"),
            Coin::new(1_000_000_000_000, "uatom"),
        ];
        let owner = app.init_account(&initial_balance).unwrap();

        // create pools
        gamm.create_basic_pool(
            &[Coin::new(1_000, "uion"), Coin::new(1_000, "uosmo")],
            &owner,
        )
        .unwrap();
        gamm.create_basic_pool(
            &[Coin::new(1_000, "uatom"), Coin::new(1_000, "uosmo")],
            &owner,
        )
        .unwrap();
        gamm.create_basic_pool(
            &[Coin::new(1_000, "uatom"), Coin::new(1_000, "uion")],
            &owner,
        )
        .unwrap();

        let code_id = wasm
            .store_code(&get_wasm(), None, &owner)
            .unwrap()
            .data
            .code_id;

        let contract_address = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    owner: owner.address(),
                },
                Some(&owner.address()),
                None,
                &[],
                &owner,
            )
            .unwrap()
            .data
            .address;

        TestEnv {
            app,
            contract_address,
            owner,
        }
    }
}

fn get_wasm() -> Vec<u8> {
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("swaprouter.wasm");
    std::fs::read(wasm_path).unwrap()
}
