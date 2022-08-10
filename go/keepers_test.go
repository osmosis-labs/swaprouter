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

func (suite *KeeperTestSuite) SetupTest() {
	suite.Setup()

	// Fund every TestAcc with 1_000_000 uosmo, token1, and token2.
	fundAccsAmount := sdk.NewCoins(
		sdk.NewInt64Coin("uosmo", 1_000_000),
		sdk.NewInt64Coin("token1", 1_000_000),
		sdk.NewInt64Coin("token2", 1_000_000))
	for _, acc := range suite.TestAccs {
		suite.FundAcc(acc, fundAccsAmount)
	}

	// setup contract keeper
	suite.contractKeeper = wasmkeeper.NewPermissionedKeeper(suite.App.WasmKeeper, SudoAuthorizationPolicy{})

	// suite.queryClient = types.NewQueryClient(suite.QueryHelper)
	// suite.msgServer = keeper.NewMsgServerImpl(*suite.App.TokenFactoryKeeper)

	suite.bankMsgServer = bankkeeper.NewMsgServerImpl(suite.App.BankKeeper)

	// create a token1/osmo pool and a token2/osmo pool
	suite.SetupGammPoolsWithBondDenomMultiplier([]sdk.Dec{sdk.OneDec(), sdk.OneDec()})

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
