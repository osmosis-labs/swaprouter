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
			desc: "set initial route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "stake",
				OutputDenom: "token0",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "token0",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
		{
			desc: "override route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "stake",
				OutputDenom: "token0",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "token0",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
		{
			desc: "don't allow non-owner to set routes",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "stake",
				OutputDenom: "token0",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "token0",
				}},
			}},
			sender:     suite.TestAccs[1],
			expectPass: false,
		},
		{
			desc: "invalid route - output_denom doesn't match ending pool route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "stake",
				OutputDenom: "token0",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "stake",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "multi hop route setting",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "token0",
				OutputDenom: "token1",
				PoolRoute: []gammtypes.SwapAmountInRoute{
					{
						PoolId:        1,
						TokenOutDenom: "stake",
					},
					{
						PoolId:        2,
						TokenOutDenom: "token1",
					}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
	} {
		suite.Run(fmt.Sprintf("Case %s", tc.desc), func() {

			pool1, _ := suite.App.GAMMKeeper.GetPoolAndPoke(suite.Ctx, 1)
			fmt.Println(pool1.GetTotalPoolLiquidity(suite.Ctx))

			msg, err := json.Marshal(conventionalMarshaller{tc.setRouteMsg})

			suite.Require().NoError(err, "test %v", tc.desc)
			fmt.Println(string(msg))

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
