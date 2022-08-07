/**
* This file was automatically generated by cosmwasm-typescript-gen@0.3.9.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the cosmwasm-typescript-gen generate command to regenerate this file.
*/

import { CosmWasmClient, ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
export type ExecuteMsg = {
  set_route: {
    input_denom: string;
    output_denom: string;
    pool_route: SwapAmountInRoute[];
    [k: string]: unknown;
  };
};
export interface SwapAmountInRoute {
  pool_id: number;
  token_out_denom: string;
  [k: string]: unknown;
}
export interface GetOwnerResponse {
  owner: string;
  [k: string]: unknown;
}
export interface GetRouteResponse {
  pool_route: SwapAmountInRoute[];
  [k: string]: unknown;
}
export interface InstantiateMsg {
  owner: string;
  [k: string]: unknown;
}
export type QueryMsg = {
  get_owner: {
    [k: string]: unknown;
  };
} | {
  get_route: {
    input_denom: string;
    output_denom: string;
    [k: string]: unknown;
  };
};
export type Addr = string;
export interface State {
  owner: Addr;
  [k: string]: unknown;
}
export interface SwapMsgReplyState {
  original_sender: Addr;
  swap_msg: MsgSwapExactAmountIn;
  [k: string]: unknown;
}
export interface MsgSwapExactAmountIn {
  routes: SwapAmountInRoute[];
  sender: string;
  token_in?: Coin | null;
  token_out_min_amount: string;
  [k: string]: unknown;
}
export interface Coin {
  amount: string;
  denom: string;
  [k: string]: unknown;
}
export interface SwaprouterReadOnlyInterface {
  contractAddress: string;
  getOwner: () => Promise<GetOwnerResponse>;
  getRoute: ({
    inputDenom,
    outputDenom
  }: {
    inputDenom: string;
    outputDenom: string;
  }) => Promise<GetRouteResponse>;
}
export class SwaprouterQueryClient implements SwaprouterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.getOwner = this.getOwner.bind(this);
    this.getRoute = this.getRoute.bind(this);
  }

  getOwner = async (): Promise<GetOwnerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_owner: {}
    });
  };
  getRoute = async ({
    inputDenom,
    outputDenom
  }: {
    inputDenom: string;
    outputDenom: string;
  }): Promise<GetRouteResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_route: {
        input_denom: inputDenom,
        output_denom: outputDenom
      }
    });
  };
}
export interface SwaprouterInterface extends SwaprouterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  setRoute: ({
    inputDenom,
    outputDenom,
    poolRoute
  }: {
    inputDenom: string;
    outputDenom: string;
    poolRoute: SwapAmountInRoute[];
  }, fee?: number | StdFee | "auto", memo?: string, funds?: readonly Coin[]) => Promise<ExecuteResult>;
}
export class SwaprouterClient extends SwaprouterQueryClient implements SwaprouterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.setRoute = this.setRoute.bind(this);
  }

  setRoute = async ({
    inputDenom,
    outputDenom,
    poolRoute
  }: {
    inputDenom: string;
    outputDenom: string;
    poolRoute: SwapAmountInRoute[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: readonly Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_route: {
        input_denom: inputDenom,
        output_denom: outputDenom,
        pool_route: poolRoute
      }
    }, fee, memo, funds);
  };
}