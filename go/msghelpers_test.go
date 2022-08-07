package swaprouter_test

import (
	gammtypes "github.com/osmosis-labs/osmosis/v10/x/gamm/types"
)

type InstantiateMsg struct {
	Denom string `json:"denom"`
}

type ExecuteMsg struct {
	SetRoute *SetRoute `json:"set_route,omitempty"`
}

type SetRoute struct {
	InputDenom  string                        `json:"input_denom"`
	OutputDenom string                        `json:"output_denom"`
	PoolRoute   []gammtypes.SwapAmountInRoute `json:"pool_route"`
}

type QueryMsg struct {
	GetOwner *GetOwner `json:"get_owner,omitempty"`
	GetRoute *GetRoute `json:"get_route,omitempty"`
}

type GetOwner struct{}

type GetRoute struct {
	InputDenom  string `json:"input_denom"`
	OutputDenom string `json:"output_denom"`
}

// ---

type GetOwnerResponse struct {
	Owner string `json:"owner"`
}

type GetRouteResponse struct {
	PoolRoute []gammtypes.SwapAmountInRoute `json:"pool_route"`
}
