mod common;
use common::*;
use cosmwasm_std::Coin;
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use osmosis_testing::account::Account;
use swaprouter::msg::{ExecuteMsg, GetRouteResponse, QueryMsg};

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
    ($test_name:ident should succeed, sender = @owner, msg = $msg:expr) => {
        #[test]
        fn $test_name() {
            let (app, contract_address, owner) = setup_test_env();
            let res = app.execute_contract(&owner.address(), &contract_address, &$msg, &[]);
            assert!(res.is_ok(), "{}", res.unwrap_err());

            // check if set route can be queried correctly
            if let ExecuteMsg::SetRoute {
                input_denom,
                output_denom,
                ..
            } = $msg
            {
                let query = QueryMsg::GetRoute {
                    input_denom: input_denom.clone(),
                    output_denom: output_denom.clone(),
                };

                let _: GetRouteResponse = app
                    .query_contract(&contract_address, &query)
                    .expect(&format!("Query with `{:?}` must succeed", query));
            } else {
                panic!("ExecuteMsg must be `SetRoute`");
            }
        }
    };

    ($test_name:ident should failed_with $err:expr, sender = @$sender:ident, msg = $msg:expr) => {
        #[test]
        fn $test_name() {
            let (app, contract_address, owner) = setup_test_env();

            let sender = if stringify!($sender) == "owner" {
                owner
            } else {
                let initial_balance = [
                    Coin::new(1_000_000_000_000, "uosmo"),
                    Coin::new(1_000_000_000_000, "uion"),
                    Coin::new(1_000_000_000_000, "uatom"),
                ];
                app.init_account(&initial_balance)
            };

            let res = app.execute_contract(&sender.address(), &contract_address, &$msg, &[]);
            assert_eq!(res.unwrap_err(), $err);
        }
    };
}
