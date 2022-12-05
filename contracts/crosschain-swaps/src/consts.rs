use phf::phf_map;

// Msg Reply IDs
pub const SWAP_REPLY_ID: u64 = 1u64;
pub const FORWARD_REPLY_ID: u64 = 2u64;

// IBC timeout
pub const PACKET_LIFETIME: u64 = 86400u64;

// Known channels
pub const CHANNEL_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "cosmoshub" => "channel-0",
    "juno" => "channel-42",
    "axelar" => "channel-208",
};
