// Tachyon Oracle Gossip - Full Solana Clone
// Adapted from Solana's gossip protocol for oracle networks

#![allow(clippy::arithmetic_side_effects)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// Re-export from Solana SDK (we already have these)
pub use solana_sdk::pubkey::Pubkey;
pub use solana_sdk::signature::{Keypair, Signature, Signer};

// Core modules
pub mod gossip_error;
pub mod crds_gossip_error;
pub mod crds_entry;
pub mod crds_value;
pub mod crds;
pub mod crds_data;
pub mod crds_shards;
pub mod contact_info;
pub mod legacy_contact_info;

// Protocol
pub mod protocol;
pub mod crds_gossip;
pub mod crds_gossip_push;
pub mod crds_gossip_pull;
pub mod ping_pong;
pub mod received_cache;

// Optimization
pub mod weighted_shuffle;
pub mod push_active_set;

// Main coordinator
pub mod cluster_info;
pub mod cluster_info_metrics;
pub mod gossip_service;

// Epochs (for price cycles)
pub mod epoch_slots;
pub mod epoch_specs;
pub mod restart_crds_values;

// Duplicate detection (for malicious oracles)
pub mod duplicate_shred;
pub mod duplicate_shred_handler;
pub mod duplicate_shred_listener;

// Deprecated/legacy
mod deprecated;

// Re-exports for convenience
pub use cluster_info::ClusterInfo;
pub use contact_info::ContactInfo;
pub use crds::Crds;
pub use crds_value::CrdsValue;
pub use gossip_service::GossipService;

