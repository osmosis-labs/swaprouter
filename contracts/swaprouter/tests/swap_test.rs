mod common;
use common::*;
use cosmwasm_std::Coin;
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use osmosis_testing::account::Account;
use swaprouter::msg::ExecuteMsg;

test_swap!(
    try_swap_for_correct_route
    should succeed,

    msg = ExecuteMsg::Swap {
        input_coin: Coin::new(1000, "uosmo"),
        output_denom: "uion".to_string(),
        minimum_output_amount: 1u128.into()
    },
    funds: [
        Coin::new(1000, "uosmo")
    ]
);

test_swap!(
    not_enough_attached_funds_to_swap should failed_with
    "Insufficient Funds: execute wasm contract failed",

    msg = ExecuteMsg::Swap {
        input_coin: Coin::new(1000, "uosmo"),
        output_denom: "uion".to_string(),
        minimum_output_amount: 1u128.into()
    },
    funds: [
        Coin::new(10, "uosmo")
    ]
);

test_swap!(
    wrong_denom_attached_funds should failed_with
    "Insufficient Funds: execute wasm contract failed",

    msg = ExecuteMsg::Swap {
        input_coin: Coin::new(1000, "uosmo"),
        output_denom: "uion".to_string(),
        minimum_output_amount: 1u128.into()
    },
    funds: [
        Coin::new(10, "uion")
    ]
);

test_swap!(
    minimum_output_amount_too_high should failed_with
    "dispatch: submessages: uion token is lesser than min amount: calculated amount is lesser than min amount",

    msg = ExecuteMsg::Swap {
        input_coin: Coin::new(1000, "uosmo"),
        output_denom: "uion".to_string(),
        minimum_output_amount: 1000000000000000000000000u128.into()
    },
    funds: [
        Coin::new(1000, "uosmo")
    ]
);

test_swap!(
    non_existant_route should failed_with
    "alloc::vec::Vec<osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute> not found: execute wasm contract failed",

    msg = ExecuteMsg::Swap {
        input_coin: Coin::new(1000, "uion"),
        output_denom: "uosmo".to_string(),
        minimum_output_amount: 1000000000000000000000000u128.into()
    },
    funds: [
        Coin::new(1000, "uion")
    ]
);

// ======= helpers ========

#[macro_export]
macro_rules! test_swap {
    ($test_name:ident should succeed, msg = $msg:expr, funds: $funds:expr) => {
        #[test]
        fn $test_name() {
            let res = setup_and_swap(&$msg, &$funds);
            assert!(res.is_ok(), "{}", res.unwrap_err());
        }
    };
    ($test_name:ident should failed_with $err:expr, msg = $msg:expr, funds: $funds:expr) => {
        #[test]
        fn $test_name() {
            let res = setup_and_swap(&$msg, &$funds);
            assert_eq!(res.unwrap_err(), $err);
        }
    };
}

fn setup_and_swap(msg: &ExecuteMsg, funds: &[Coin]) -> Result<String, String> {
    let (app, contract_address, owner) = setup_test_env();

    let initial_balance = [
        Coin::new(1_000_000_000_000, "uosmo"),
        Coin::new(1_000_000_000_000, "uion"),
        Coin::new(1_000_000_000_000, "uatom"),
    ];

    let non_owner = app.init_account(&initial_balance);
    let set_route_msg = ExecuteMsg::SetRoute {
        input_denom: "uosmo".to_string(),
        output_denom: "uion".to_string(),
        pool_route: vec![
            SwapAmountInRoute {
                pool_id: 2,
                token_out_denom: "uatom".to_string(),
            },
            SwapAmountInRoute {
                pool_id: 3,
                token_out_denom: "uion".to_string(),
            },
        ],
    };
    app.execute_contract(&owner.address(), &contract_address, &set_route_msg, &[])
        .expect("Setup route fixture must always succeed");

    app.execute_contract(&non_owner.address(), &contract_address, &msg, funds)
}
