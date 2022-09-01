use std::path::PathBuf;

use cosmwasm_std::Coin;
use serial_test::serial;

use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use osmosis_testing::{
    account::{Account, SigningAccount},
    app::App,
};
use swaprouter::msg::{ExecuteMsg, GetRouteResponse, InstantiateMsg, QueryMsg};

test_set_route!(
    set_initial_route_by_non_owner
    should failed_with "Unauthorized: execute wasm contract failed",

    sender = @non_owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uion".to_string(),
        pool_route: vec![SwapAmountInRoute {
            pool_id: 1,
            token_out_denom: "uion".to_string(),
        }],
    }
);

test_set_route!(
    set_initial_route_by_owner
    should succeed,

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uion".to_string(),
        pool_route: vec![SwapAmountInRoute {
            pool_id: 1,
            token_out_denom: "uion".to_string(),
        }],
    }
);

test_set_route!(
    override_route_with_multi_hop
    should succeed,

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uion".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 2, // uatom/uosmo
                token_out_denom: "uatom".to_string(),
            },
            SwapAmountInRoute {
                pool_id: 3, // uatom/uion
                token_out_denom: "uion".to_string(),
            }
        ],
    }
);

test_set_route!(
    output_denom_that_does_not_ending_pool_route
    should failed_with
    r#"Invalid Pool Route: "last denom doesn't match": execute wasm contract failed"#,
    // r#"Invalid Pool Route: "denom uosmo is not in pool id 1": execute wasm contract failed"#,

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uion".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 1, // uosmo/uion
                token_out_denom: "uosmo".to_string(),
            },
        ],
    }
);

test_set_route!(
    pool_does_not_have_input_asset
    should failed_with
    r#"Invalid Pool Route: "denom uatom is not in pool id 1": execute wasm contract failed"#,

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uatom".to_string(),
        output_denom: "uion".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 1, // uosmo/uion
                token_out_denom: "uion".to_string(),
            },
        ],
    }
);

test_set_route!(
    pool_does_not_have_output_asset
    should failed_with
    r#"Invalid Pool Route: "denom uosmo is not in pool id 1": execute wasm contract failed"#,
    // confusing error message from chain, should state that:
    // > `denom uatom is not in pool id 1": execute wasm contract failed`
    // instead.

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uatom".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 1, // uosmo/uion
                token_out_denom: "uatom".to_string(),
            },
        ],
    }
);

test_set_route!(
    intermediary_pool_does_not_have_output_asset
    should failed_with
    r#"Invalid Pool Route: "denom uosmo is not in pool id 1": execute wasm contract failed"#,
    // confusing error message from chain, should state that:
    // > `denom foocoin is not in pool id 1": execute wasm contract failed`
    // instead.

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uatom".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 1, // uosmo/uion
                token_out_denom: "foocoin".to_string(),
            },
            SwapAmountInRoute {
                pool_id: 2, // uatom/uosmo
                token_out_denom: "uatom".to_string(),
            },
        ],
    }
);

test_set_route!(
    intermediary_pool_does_not_have_input_asset
    should failed_with
    r#"Invalid Pool Route: "denom uion is not in pool id 2": execute wasm contract failed"#,

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uatom".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 1, // uosmo/uion
                token_out_denom: "uion".to_string(),
            },
            SwapAmountInRoute {
                pool_id: 2, // uatom/uosmo
                token_out_denom: "uatom".to_string(),
            },
        ],
    }
);

test_set_route!(
    non_existant_pool
    should failed_with
    r#"Invalid Pool Route: "denom uosmo is not in pool id 3": execute wasm contract failed"#,

    sender = @owner,
    msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uatom".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 3, // uatom/uion
                token_out_denom: "uion".to_string(),
            },
        ],
    }
);

// ======= helpers ========

#[macro_export]
macro_rules! test_set_route {
    ($test_name:ident should failed_with $err:expr, sender = @owner, msg = $msg:expr) => {
        #[test]
        #[serial]
        fn $test_name() {
            let (app, contract_address, owner) = setup_test();
            let res = app.execute_contract(&owner.address(), &contract_address, &$msg, &[]);
            assert_eq!(res.unwrap_err(), $err);
        }
    };
    ($test_name:ident should succeed, sender = @owner, msg = $msg:expr) => {
        #[test]
        #[serial]
        fn $test_name() {
            let (app, contract_address, owner) = setup_test();
            let res = app.execute_contract(&owner.address(), &contract_address, &$msg, &[]);
            assert!(res.is_ok(), "{}", res.unwrap_err());

            // if let ExecuteMsg::SetRoute {
            //     input_denom,
            //     output_denom,
            //     ..
            // } = $msg {
            //     let query_res: GetRouteResponse = app.query_contract(
            //         &contract_address,
            //         &QueryMsg::GetRoute {
            //             input_denom,
            //             output_denom,
            //         },
            //     );

            //     dbg!(query_res);
            // } else {
            //     panic!("ExecuteMsg must be `SetRoute`");
            // }
        }
    };

    ($test_name:ident should failed_with $err:expr, sender = @non_owner, msg = $msg:expr) => {
        #[test]
        fn $test_name() {
            let (app, contract_address, _) = setup_test();

            let initial_balance = [
                Coin::new(1_000_000_000_000, "uosmo"),
                Coin::new(1_000_000_000_000, "uion"),
                Coin::new(1_000_000_000_000, "uatom"),
            ];
            let non_owner = app.init_account(&initial_balance);

            let res = app.execute_contract(&non_owner.address(), &contract_address, &$msg, &[]);
            assert_eq!(res.unwrap_err(), $err);
        }
    };
}

fn setup_test() -> (App, String, SigningAccount) {
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

    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("swaprouter.wasm");

    let code_id = app
        .store_code_from_path(&owner.address(), wasm_path)
        .unwrap();

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
