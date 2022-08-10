package swaprouter_test

import (
	"fmt"
	"io/ioutil"
	"testing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/suite"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/types"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"

	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"

	"github.com/osmosis-labs/osmosis/v10/app/apptesting"
	"github.com/osmosis-labs/osmosis/v10/x/gamm/pool-models/balancer"
	gammtypes "github.com/osmosis-labs/osmosis/v10/x/gamm/types"
)

var (
	wasmFile = "../target/wasm32-unknown-unknown/release/swaprouter.wasm"
	// wasmFile = "../artifacts/swaprouter.wasm"
)

type KeeperTestSuite struct {
	apptesting.KeeperTestHelper

	queryClient    types.QueryClient
	msgServer      types.MsgServer
	contractKeeper wasmtypes.ContractOpsKeeper
	bankMsgServer  banktypes.MsgServer

	codeId uint64
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

type SudoAuthorizationPolicy struct{}

func (p SudoAuthorizationPolicy) CanCreateCode(config wasmtypes.AccessConfig, actor sdk.AccAddress) bool {
	return true
}

func (p SudoAuthorizationPolicy) CanInstantiateContract(config wasmtypes.AccessConfig, actor sdk.AccAddress) bool {
	return true
}

func (p SudoAuthorizationPolicy) CanModifyContract(admin, actor sdk.AccAddress) bool {
	return true
}

func (suite *KeeperTestSuite) CreatePool(initialLiquidity sdk.Coins) gammtypes.PoolI {
	var poolAssets []balancer.PoolAsset

	for _, asset := range initialLiquidity {
		poolAssets = append(poolAssets, balancer.PoolAsset{
			Weight: sdk.NewInt(1),
			Token:  asset,
		})
	}

	poolParams := balancer.PoolParams{
		SwapFee: sdk.NewDecWithPrec(1, 2),
		ExitFee: sdk.NewDecWithPrec(1, 2),
	}

	msg := balancer.NewMsgCreateBalancerPool(suite.TestAccs[0], poolParams, poolAssets, "")

	poolId, err := suite.App.GAMMKeeper.CreatePool(suite.Ctx, msg)
	suite.Require().NoError(err)

	pool, err := suite.App.GAMMKeeper.GetPoolAndPoke(suite.Ctx, poolId)
	suite.Require().NoError(err)

	return pool
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.Setup()

	// Fund every TestAcc with 1_000_000_000_000 uosmo, uion, and uatom.
	fundAccsAmount := sdk.NewCoins(
		sdk.NewInt64Coin("uosmo", 1_000_000_000_000),
		sdk.NewInt64Coin("uion", 1_000_000_000_000),
		sdk.NewInt64Coin("uatom", 1_000_000_000_000))
	for _, acc := range suite.TestAccs {
		suite.FundAcc(acc, fundAccsAmount)
	}

	// setup contract keeper
	suite.contractKeeper = wasmkeeper.NewPermissionedKeeper(suite.App.WasmKeeper, SudoAuthorizationPolicy{})

	// suite.queryClient = types.NewQueryClient(suite.QueryHelper)
	// suite.msgServer = keeper.NewMsgServerImpl(*suite.App.TokenFactoryKeeper)

	suite.bankMsgServer = bankkeeper.NewMsgServerImpl(suite.App.BankKeeper)

	// create a token1/stake pool and a token2/stake pool
	// suite.SetupGammPoolsWithBondDenomMultiplier([]sdk.Dec{sdk.OneDec(), sdk.OneDec()})

	// create pool 1 as uosmo/uion
	suite.CreatePool(sdk.NewCoins(sdk.NewInt64Coin("uosmo", 1_000), sdk.NewInt64Coin("uion", 1_000)))
	// create pool 2 as uosmo/uatom
	suite.CreatePool(sdk.NewCoins(sdk.NewInt64Coin("uosmo", 1_000), sdk.NewInt64Coin("uatom", 1_000)))
	// create pool 3 as uatom/uion
	suite.CreatePool(sdk.NewCoins(sdk.NewInt64Coin("uion", 1_000), sdk.NewInt64Coin("uatom", 1_000)))

	// upload wasm code
	wasmCode, err := ioutil.ReadFile(wasmFile)
	suite.Require().NoError(err)
	suite.codeId, err = suite.contractKeeper.Create(suite.Ctx, suite.TestAccs[0], wasmCode, nil)
	suite.Require().NoError(err)
}

func (suite *KeeperTestSuite) InstantiateContract(owner string) (contractAddr sdk.AccAddress) {
	instantateMsg := []byte(fmt.Sprintf("{ \"owner\": \"%v\" }", owner))

	contractAddr, _, err := suite.contractKeeper.Instantiate(suite.Ctx, suite.codeId, suite.TestAccs[0], suite.TestAccs[0], instantateMsg, "", sdk.NewCoins())
	suite.Require().NoError(err)

	return contractAddr
}
