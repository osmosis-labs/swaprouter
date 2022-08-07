/**
* This file was automatically generated by cosmwasm-typescript-gen@0.3.9.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the cosmwasm-typescript-gen generate command to regenerate this file.
*/
import { CosmWasmClient, ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
export declare type ExecuteMsg = {
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
export declare type QueryMsg = {
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
export declare type Addr = string;
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
    getRoute: ({ inputDenom, outputDenom }: {
        inputDenom: string;
        outputDenom: string;
    }) => Promise<GetRouteResponse>;
}
export declare class SwaprouterQueryClient implements SwaprouterReadOnlyInterface {
    client: CosmWasmClient;
    contractAddress: string;
    constructor(client: CosmWasmClient, contractAddress: string);
    getOwner: () => Promise<GetOwnerResponse>;
    getRoute: ({ inputDenom, outputDenom }: {
        inputDenom: string;
        outputDenom: string;
    }) => Promise<GetRouteResponse>;
}
export interface SwaprouterInterface extends SwaprouterReadOnlyInterface {
    contractAddress: string;
    sender: string;
    setRoute: ({ inputDenom, outputDenom, poolRoute }: {
        inputDenom: string;
        outputDenom: string;
        poolRoute: SwapAmountInRoute[];
    }, fee?: number | StdFee | "auto", memo?: string, funds?: readonly Coin[]) => Promise<ExecuteResult>;
}
export declare class SwaprouterClient extends SwaprouterQueryClient implements SwaprouterInterface {
    client: SigningCosmWasmClient;
    sender: string;
    contractAddress: string;
    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string);
    setRoute: ({ inputDenom, outputDenom, poolRoute }: {
        inputDenom: string;
        outputDenom: string;
        poolRoute: SwapAmountInRoute[];
    }, fee?: number | StdFee | "auto", memo?: string, funds?: readonly Coin[]) => Promise<ExecuteResult>;
}
//# sourceMappingURL=SwaprouterContract.d.ts.map