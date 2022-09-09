mod common;
use common::*;
use cosmwasm_std::Coin;
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use osmosis_testing::{account::Account, runner::app::App};
use osmosis_testing::runner::error::RunnerError;
use osmosis_testing::x::Module;
use osmosis_testing::x::wasm::Wasm;
use osmosis_testing::x::bank::Bank;
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
            let res = setup_and_swap(&$msg, &$funds, &check_input_decreased_and_output_increased);
            assert!(res.is_ok(), "{}", res.unwrap_err());
        }
    };
    ($test_name:ident should failed_with $err:expr, msg = $msg:expr, funds: $funds:expr) => {
        #[test]
        fn $test_name() {
            let res = setup_and_swap(&$msg, &$funds, &|_, _, _| {});
            assert_eq!(res.unwrap_err(), format!("failed to execute message; message index: 0: {}", $err));
        }
    };
}

const INITIAL_AMOUNT: u128 = 1_000_000_000_000;

fn setup_and_swap(
    msg: &ExecuteMsg,
    funds: &[Coin],
    check: &dyn Fn(&App, &str, &ExecuteMsg),
) -> Result<String, String> {
    let (app, contract_address, owner) = setup_test_env();
    let wasm = Wasm::new(&app);

    let initial_balance = [
        Coin::new(INITIAL_AMOUNT, "uosmo"),
        Coin::new(INITIAL_AMOUNT, "uion"),
        Coin::new(INITIAL_AMOUNT, "uatom"),
    ];

    let non_owner = app.init_account(&initial_balance).unwrap();
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
    wasm.execute(&contract_address,&set_route_msg,    &[], &owner)
        .expect("Setup route fixture must always succeed");

    let res = wasm.execute(&contract_address, &msg, funds, &non_owner);

    check(&app, &non_owner.address(), msg);
     res.map(|_| "".to_string()).map_err(|e| {
        match e {
            RunnerError::AppError { msg} => msg,
            _ => panic!("unexpected error")
        }
    })
}

fn check_input_decreased_and_output_increased(app: &App, sender: &str, msg: &ExecuteMsg) {
    let bank = Bank::new(app);
    let balances = bank.query_all_balances(sender, None).balances;
    if let ExecuteMsg::Swap {
        input_coin,
        output_denom,
        ..
    } = msg
    {
        let input = balances
            .iter()
            .find(|b| b.denom == input_coin.denom)
            .unwrap();
        let output = balances
            .iter()
            .find(|b| *b.denom == *output_denom.as_str())
            .unwrap();

        assert!(
            input.amount.parse::<u128>().unwrap() < INITIAL_AMOUNT,
            "Input must be decreased after swap"
        );
        assert!(
            output.amount.parse::<u128>().unwrap() > INITIAL_AMOUNT,
            "Output must be increased after swap"
        );
    } else {
        panic!("Wrong message type: Must be `ExecuteMsg::Swap`");
    }
}
