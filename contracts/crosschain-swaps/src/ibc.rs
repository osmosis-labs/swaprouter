use osmosis_std_derive::CosmwasmExt;

// We need to define the transfer here as a stargate messages because this is
// not yet supported by cosmwasm-std
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/ibc.applications.transfer.v1.MsgTransfer")]
pub struct MsgTransfer {
    #[prost(string, tag = "1")]
    pub source_port: String,
    #[prost(string, tag = "2")]
    pub source_channel: String,
    #[prost(message, optional, tag = "3")]
    pub token: ::core::option::Option<osmosis_std::types::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "4")]
    pub sender: String,
    #[prost(string, tag = "5")]
    pub receiver: String,
    #[prost(string, optional, tag = "6")]
    pub timeout_height: Option<String>,
    #[prost(uint64, optional, tag = "7")]
    pub timeout_timestamp: ::core::option::Option<u64>,
    #[prost(string, tag = "8")]
    pub memo: String,
}

// We define the response as a prost message to be able to decode the protobuf data.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTransferResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
