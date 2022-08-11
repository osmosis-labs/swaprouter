package swaprouter_test

import (
	"encoding/json"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	gammtypes "github.com/osmosis-labs/osmosis/v10/x/gamm/types"

	cwmath "github.com/cosmwasm/cosmwasm-go/std/math"
	cwtypes "github.com/cosmwasm/cosmwasm-go/std/types"
)

type setRouteMsgTest struct {
	setRouteMsg ExecuteMsg
	sender      sdk.AccAddress
	expectPass  bool
}

func (suite *KeeperTestSuite) TestSetRoutes() {
	suite.SetupTest()
	contractAddress := suite.InstantiateContract(suite.TestAccs[0].String())

	for _, tc := range []struct {
		desc        string
		setRouteMsg ExecuteMsg
		sender      sdk.AccAddress
		expectPass  bool
	}{
		{
			desc: "don't allow non-owner to set routes",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uion",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "uion",
				}},
			}},
			sender:     suite.TestAccs[1],
			expectPass: false,
		},
		{
			desc: "set initial route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uion",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "uion",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
		{
			desc: "override route with multihop",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uion",
				PoolRoute: []gammtypes.SwapAmountInRoute{
					{
						PoolId:        2,
						TokenOutDenom: "uatom",
					},
					{
						PoolId:        3,
						TokenOutDenom: "uion",
					},
				},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
		{
			desc: "invalid route - output_denom doesn't match ending pool route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uion",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "uosmo",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "invalid route - single hop - pool doesn't have input asset",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uatom",
				OutputDenom: "uion",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "uion",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "invalid route - single hop - pool doesn't have output asset",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uatom",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "uatom",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "invalid route - multi hop - intermediary pool doesn't have output asset",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uatom",
				PoolRoute: []gammtypes.SwapAmountInRoute{
					{
						PoolId:        1,
						TokenOutDenom: "foocoin",
					},
					{
						PoolId:        2,
						TokenOutDenom: "uatom",
					},
				},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "invalid route - multi hop - intermediary pool doesn't have input asset",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uatom",
				PoolRoute: []gammtypes.SwapAmountInRoute{
					{
						PoolId:        1,
						TokenOutDenom: "uion",
					},
					{
						PoolId:        2,
						TokenOutDenom: "uatom",
					},
				},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "invalid route - non existant pool",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "uatom",
				PoolRoute: []gammtypes.SwapAmountInRoute{
					{
						PoolId:        3,
						TokenOutDenom: "uion",
					},
				},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
	} {
		suite.Run(fmt.Sprintf("Case %s", tc.desc), func() {
			msg, err := json.Marshal(conventionalMarshaller{tc.setRouteMsg})
			suite.Require().NoError(err, "test %v", tc.desc)

			_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddress, tc.sender, msg, sdk.NewCoins())
			if tc.expectPass {
				suite.Require().NoError(err)

				queryMsg, err := json.Marshal(QueryMsg{GetRoute: &GetRoute{
					InputDenom:  tc.setRouteMsg.SetRoute.InputDenom,
					OutputDenom: tc.setRouteMsg.SetRoute.OutputDenom,
				}})
				suite.Require().NoError(err)

				resBz, err := suite.contractQueryKeeper.QuerySmart(suite.Ctx, contractAddress, queryMsg)
				suite.Require().NoError(err)
				var res GetRouteResponse
				json.Unmarshal(resBz, &res)

				// todo deal with snake case stuff
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestSwaps() {
	suite.SetupTest()
	contractAddress := suite.InstantiateContract(suite.TestAccs[0].String())

	// set route uosmo -> uion as multihop pool route through uatom

	msg, err := json.Marshal(conventionalMarshaller{ExecuteMsg{SetRoute: &SetRoute{
		InputDenom:  "uosmo",
		OutputDenom: "uion",
		PoolRoute: []gammtypes.SwapAmountInRoute{
			{
				PoolId:        2,
				TokenOutDenom: "uatom",
			},
			{
				PoolId:        3,
				TokenOutDenom: "uion",
			},
		},
	}}})
	suite.Require().NoError(err)
	_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddress, suite.TestAccs[0], msg, sdk.NewCoins())
	suite.Require().NoError(err)

	balances := make(map[string]sdk.Coins)

	for _, acc := range suite.TestAccs {
		balances[acc.String()] = suite.App.BankKeeper.GetAllBalances(suite.Ctx, acc)
	}

	for _, tc := range []struct {
		desc        string
		swapMsg     ExecuteMsg
		attachCoins sdk.Coins
		sender      sdk.AccAddress
		expectPass  bool
	}{
		{
			desc: "try swap for correct route",
			swapMsg: ExecuteMsg{Swap: &Swap{
				InputCoin:           cwtypes.NewCoin(cwmath.NewUint128FromUint64(1000), "uosmo"),
				OutputDenom:         "uion",
				MinimumOutputAmount: cwmath.NewUint128FromUint64(1),
			}},
			attachCoins: sdk.NewCoins(sdk.NewInt64Coin("uosmo", 1000)),
			sender:      suite.TestAccs[1],
			expectPass:  true,
		},
		{
			desc: "not enough attached coins",
			swapMsg: ExecuteMsg{Swap: &Swap{
				InputCoin:           cwtypes.NewCoin(cwmath.NewUint128FromUint64(1000), "uosmo"),
				OutputDenom:         "uion",
				MinimumOutputAmount: cwmath.NewUint128FromUint64(1000),
			}},
			attachCoins: sdk.NewCoins(sdk.NewInt64Coin("uosmo", 10)),
			sender:      suite.TestAccs[1],
			expectPass:  false,
		},
		{
			desc: "wrong denom attached coins",
			swapMsg: ExecuteMsg{Swap: &Swap{
				InputCoin:           cwtypes.NewCoin(cwmath.NewUint128FromUint64(1000), "uosmo"),
				OutputDenom:         "uion",
				MinimumOutputAmount: cwmath.NewUint128FromUint64(1000),
			}},
			attachCoins: sdk.NewCoins(sdk.NewInt64Coin("uion", 10)),
			sender:      suite.TestAccs[1],
			expectPass:  false,
		},
		{
			desc: "minimum_output_amount too low",
			swapMsg: ExecuteMsg{Swap: &Swap{
				InputCoin:           cwtypes.NewCoin(cwmath.NewUint128FromUint64(1000), "uosmo"),
				OutputDenom:         "uion",
				MinimumOutputAmount: cwmath.NewUint128FromUint64(1_000_000_000_000_000),
			}},
			attachCoins: sdk.NewCoins(sdk.NewInt64Coin("uosmo", 1000)),
			sender:      suite.TestAccs[1],
			expectPass:  false,
		},
		{
			desc: "non existant route",
			swapMsg: ExecuteMsg{Swap: &Swap{
				InputCoin:           cwtypes.NewCoin(cwmath.NewUint128FromUint64(1000), "uion"),
				OutputDenom:         "uosmo",
				MinimumOutputAmount: cwmath.NewUint128FromUint64(1000),
			}},
			attachCoins: sdk.NewCoins(sdk.NewInt64Coin("uion", 1000)),
			sender:      suite.TestAccs[1],
			expectPass:  false,
		},
	} {
		suite.Run(fmt.Sprintf("Case %s", tc.desc), func() {
			msg, err := json.Marshal(conventionalMarshaller{tc.swapMsg})
			fmt.Println(string(msg))

			suite.Require().NoError(err, "test %v", tc.desc)

			_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddress, tc.sender, msg, tc.attachCoins)
			if tc.expectPass {
				suite.Require().NoError(err)

				newBalances := suite.App.BankKeeper.GetAllBalances(suite.Ctx, tc.sender)

				// make sure input denom balance went down
				suite.Require().True(newBalances.AmountOf(tc.swapMsg.Swap.InputCoin.Denom).LT(balances[tc.sender.String()].AmountOf(tc.swapMsg.Swap.InputCoin.Denom)))
				// make sure output denom balance went up
				suite.Require().True(newBalances.AmountOf(tc.swapMsg.Swap.OutputDenom).GT(balances[tc.sender.String()].AmountOf(tc.swapMsg.Swap.OutputDenom)))

				// update balances
				balances[tc.sender.String()] = newBalances
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
