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
				InputDenom:  "uosmo",
				OutputDenom: "token1",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "token1",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
		{
			desc: "override route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "token1",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "token1",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
		{
			desc: "don't allow non-owner to set routes",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "token1",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "token1",
				}},
			}},
			sender:     suite.TestAccs[1],
			expectPass: false,
		},
		{
			desc: "invalid route - output_denom doesn't match ending pool route",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "uosmo",
				OutputDenom: "token1",
				PoolRoute: []gammtypes.SwapAmountInRoute{{
					PoolId:        1,
					TokenOutDenom: "uosmo",
				}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: false,
		},
		{
			desc: "multi hop route setting",
			setRouteMsg: ExecuteMsg{SetRoute: &SetRoute{
				InputDenom:  "token1",
				OutputDenom: "token2",
				PoolRoute: []gammtypes.SwapAmountInRoute{
					{
						PoolId:        1,
						TokenOutDenom: "uosmo",
					},
					{
						PoolId:        2,
						TokenOutDenom: "token2",
					}},
			}},
			sender:     suite.TestAccs[0],
			expectPass: true,
		},
	} {
		suite.Run(fmt.Sprintf("Case %s", tc.desc), func() {

			msg, err := json.Marshal(tc.setRouteMsg)
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

// func (suite *KeeperTestSuite) TestFreeze() {
// 	for _, tc := range []struct {
// 		desc       string
// 		frozen     bool
// 		bankMsg    func(denom string) *banktypes.MsgSend
// 		expectPass bool
// 	}{
// 		{
// 			desc:   "if contract is not frozen, should allow transfer of the denom",
// 			frozen: false,
// 			bankMsg: func(denom string) *banktypes.MsgSend {
// 				return banktypes.NewMsgSend(suite.TestAccs[0], suite.TestAccs[1], sdk.NewCoins(sdk.NewInt64Coin(denom, 1)))
// 			},
// 			expectPass: true,
// 		},
// 		{
// 			desc:   "if contract is not frozen, should allow transfer of multidenom",
// 			frozen: false,
// 			bankMsg: func(denom string) *banktypes.MsgSend {
// 				return banktypes.NewMsgSend(suite.TestAccs[0], suite.TestAccs[1], sdk.NewCoins(
// 					sdk.NewInt64Coin(denom, 1),
// 					sdk.NewInt64Coin("uosmo", 1),
// 				))
// 			},
// 			expectPass: true,
// 		},
// 		{
// 			desc:   "if contract is frozen, should not transfer of the denom",
// 			frozen: true,
// 			bankMsg: func(denom string) *banktypes.MsgSend {
// 				return banktypes.NewMsgSend(suite.TestAccs[0], suite.TestAccs[1], sdk.NewCoins(
// 					sdk.NewInt64Coin(denom, 1),
// 				))
// 			},
// 			expectPass: false,
// 		},
// 		{
// 			desc:   "if contract is frozen, should allow transfer of a different denom",
// 			frozen: true,
// 			bankMsg: func(denom string) *banktypes.MsgSend {
// 				return banktypes.NewMsgSend(suite.TestAccs[0], suite.TestAccs[1], sdk.NewCoins(
// 					sdk.NewInt64Coin("uosmo", 1),
// 				))
// 			},
// 			expectPass: true,
// 		},

// 		{
// 			desc:   "if contract is frozen, should not transaction of multidenom",
// 			frozen: true,
// 			bankMsg: func(denom string) *banktypes.MsgSend {
// 				return banktypes.NewMsgSend(suite.TestAccs[0], suite.TestAccs[1], sdk.NewCoins(
// 					sdk.NewInt64Coin(denom, 1),
// 					sdk.NewInt64Coin("uosmo", 1),
// 				))
// 			},
// 			expectPass: false,
// 		},
// 	} {
// 		suite.Run(fmt.Sprintf("Case %s", tc.desc), func() {
// 			// setup test
// 			suite.SetupTest()
// 			denom, contractAddr := suite.CreateTokenAndContract()

// 			// give mint permissions to testAcc0
// 			setMinterMsg, err := json.Marshal(ExecuteMsg{SetMinter: &SetMinter{Address: suite.TestAccs[0].String(), Allowance: stdMath.MaxUint128()}})
// 			suite.Require().NoError(err, "test %v", tc.desc)

// 			_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddr, suite.TestAccs[0], setMinterMsg, sdk.NewCoins())
// 			suite.Require().NoError(err, "test %v", tc.desc)

// 			// mint 100000 coins to testAcc0
// 			mintMsg, err := json.Marshal(ExecuteMsg{Mint: &Mint{ToAddress: suite.TestAccs[0].String(), Amount: stdMath.NewUint128FromUint64(1000000)}})
// 			suite.Require().NoError(err, "test %v", tc.desc)
// 			_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddr, suite.TestAccs[0], mintMsg, sdk.NewCoins())
// 			suite.Require().NoError(err, "test %v", err)

// 			// give testAcc0 freeze permissions
// 			setFreezerMsg, err := json.Marshal(ExecuteMsg{SetFreezer: &SetFreezer{Address: suite.TestAccs[0].String(), Status: true}})
// 			suite.Require().NoError(err, "test %v", tc.desc)
// 			_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddr, suite.TestAccs[0], setFreezerMsg, sdk.NewCoins())
// 			suite.Require().NoError(err, "test %v", tc.desc)

// 			// if should freeze
// 			if tc.frozen {
// 				// freeze contract
// 				freezeMsg, err := json.Marshal(ExecuteMsg{Freeze: &Freeze{Status: true}})
// 				_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddr, suite.TestAccs[0], freezeMsg, sdk.NewCoins())
// 				suite.Require().NoError(err, "test %v", tc.desc)
// 			} else {
// 				freezeMsg, err := json.Marshal(ExecuteMsg{Freeze: &Freeze{Status: false}})
// 				_, err = suite.contractKeeper.Execute(suite.Ctx, contractAddr, suite.TestAccs[0], freezeMsg, sdk.NewCoins())
// 				suite.Require().NoError(err, "test %v", tc.desc)
// 			}

// 			// // TODO: use query to make sure asset is frozen

// 			_, err = suite.bankMsgServer.Send(sdk.WrapSDKContext(suite.Ctx),
// 				tc.bankMsg(denom),
// 			)

// 			if tc.expectPass {
// 				suite.Require().NoError(err, "test: %v", tc.desc)
// 			} else {
// 				suite.Require().Error(err, "test: %v", tc.desc)
// 			}
// 		})
// 	}
// }
