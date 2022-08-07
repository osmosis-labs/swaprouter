use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, DepsMut};
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;

use crate::contract;
use crate::msg::{ExecuteMsg, GetOwnerResponse, GetRouteResponse, InstantiateMsg, QueryMsg};

static CREATOR_ADDRESS: &str = "creator";

// test helper
#[allow(unused_assignments)]
fn initialize_contract(deps: DepsMut) -> Addr {
    let msg = InstantiateMsg {
        owner: String::from(CREATOR_ADDRESS),
    };
    let info = mock_info(CREATOR_ADDRESS, &[]);

    // instantiate with enough funds provided should succeed
    contract::instantiate(deps, mock_env(), info.clone(), msg).unwrap();

    info.sender
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let owner = initialize_contract(deps.as_mut());

    // it worked, let's query the state
    let res: GetOwnerResponse =
        from_binary(&contract::query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap())
            .unwrap();
    assert_eq!(owner, res.owner);
}

#[test]
fn set_routes() {
    let mut deps = mock_dependencies();

    let owner = initialize_contract(deps.as_mut());

    let pool_route = vec![SwapAmountInRoute {
        pool_id: 2,
        token_out_denom: String::from("uion"),
    }];

    // set the route
    contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(owner.as_str(), &[]),
        ExecuteMsg::SetRoute {
            input_denom: String::from("uosmo"),
            output_denom: String::from("uion"),
            pool_route: pool_route.clone(),
        },
    )
    .unwrap();

    // query the route
    let res: GetRouteResponse = from_binary(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRoute {
                input_denom: String::from("uosmo"),
                output_denom: String::from("uion"),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res.pool_route, pool_route);
}
