package swaprouter_test

import (
	"encoding/json"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	gammtypes "github.com/osmosis-labs/osmosis/v10/x/gamm/types"
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

				// TODO: figure out how to do queries
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
