package swaprouter_test

import (
	"bytes"
	"encoding/json"
	"regexp"

	gammtypes "github.com/osmosis-labs/osmosis/v10/x/gamm/types"
)

// Regexp definitions
var keyMatchRegex = regexp.MustCompile(`\"(\w+)\":`)
var wordBarrierRegex = regexp.MustCompile(`(\w)([A-Z])`)

type conventionalMarshaller struct {
	Value interface{}
}

func (c conventionalMarshaller) MarshalJSON() ([]byte, error) {
	marshalled, err := json.Marshal(c.Value)

	converted := keyMatchRegex.ReplaceAllFunc(
		marshalled,
		func(match []byte) []byte {
			return bytes.ToLower(wordBarrierRegex.ReplaceAll(
				match,
				[]byte(`${1}_${2}`),
			))
		},
	)

	return converted, err
}

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
