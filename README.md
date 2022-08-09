# Swaprouter
---
Swaprouter helps contract developers figure out which routes their automated swaps should be using on-chain.

The swaps provided by swaprouter are not guaranteed to be the most efficient.

Deployed on testnet: osmo120en3ww06t8s7z0tmfvjqdzp959phvcwwghg59czwusxupdf4vwq5p650p
Routes in the testnet routing table:
 - OSMO -> ION
 - ION -> OSMO

## Usage with rust:
```rs
let route_query = QueryRequest::Wasm(WasmQuery::Smart {
    contract_addr: routing_table_addr,
    msg: to_binary(&swaprouter::msg::QueryMsg::GetRoute {
        input_denom: config.source_denom.clone(),
        output_denom: config.destination_denom.clone(),
    })?,
});

let route: swaprouter::msg::GetRouteResponse = deps.querier.query(&route_query)?;
```

