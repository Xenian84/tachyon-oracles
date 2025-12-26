use anchor_lang::prelude::*;

/// Global configuration for the oracle system
#[account]
pub struct ConfigAccount {
    /// Admin authority
    pub admin: Pubkey,
    /// Fee in lamports for each update
    pub update_fee_lamports: u64,
    /// Relayer cut in basis points (0-10000)
    pub relayer_cut_bps: u16,
    /// Minimum number of publishers required for quorum
    pub min_publishers: u8,
    /// Maximum age of data in seconds before considered stale
    pub max_age_sec: u32,
    /// Number of assets registered
    pub asset_count: u16,
    /// PDA bump
    pub bump: u8,
}

impl ConfigAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8 +  // update_fee_lamports
        2 +  // relayer_cut_bps
        1 +  // min_publishers
        4 +  // max_age_sec
        2 +  // asset_count
        1;   // bump
}

/// Publisher account for oracle data providers
#[account]
pub struct PublisherAccount {
    /// Publisher's public key
    pub publisher: Pubkey,
    /// Amount staked (v0: recorded but not enforced)
    pub staked_amount: u64,
    /// Whether the publisher is active
    pub is_active: bool,
    /// PDA bump
    pub bump: u8,
}

impl PublisherAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // publisher
        8 +  // staked_amount
        1 +  // is_active
        1;   // bump
}

/// Price feed account for a specific asset
#[account]
pub struct PriceFeedAccount {
    /// Asset identifier hash (sha256 of asset string like "BTC/USD")
    pub asset_id: [u8; 32],
    /// Price in fixed-point i64 (e.g., * 1e6)
    pub price_i64: i64,
    /// Confidence interval in fixed-point i64
    pub conf_i64: i64,
    /// Unix timestamp of publication
    pub publish_time: i64,
    /// Last update slot
    pub last_update_slot: u64,
    /// PDA bump
    pub bump: u8,
}

impl PriceFeedAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // asset_id
        8 +  // price_i64
        8 +  // conf_i64
        8 +  // publish_time
        8 +  // last_update_slot
        1;   // bump
}

/// Signed message from a publisher
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SignedMessage {
    /// Publisher's public key
    pub publisher: Pubkey,
    /// Asset ID hash
    pub asset_id_hash: [u8; 32],
    /// Price in fixed-point
    pub price_i64: i64,
    /// Confidence in fixed-point
    pub conf_i64: i64,
    /// Publication timestamp
    pub publish_time: i64,
    /// Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl SignedMessage {
    /// Get canonical message bytes for signature verification
    pub fn get_message_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(56);
        bytes.extend_from_slice(&self.asset_id_hash);
        bytes.extend_from_slice(&self.price_i64.to_le_bytes());
        bytes.extend_from_slice(&self.conf_i64.to_le_bytes());
        bytes.extend_from_slice(&self.publish_time.to_le_bytes());
        bytes
    }
}

/// Fee vault PDA for collecting fees
#[account]
pub struct FeeVault {
    /// Total fees collected
    pub total_collected: u64,
    /// PDA bump
    pub bump: u8,
}

impl FeeVault {
    pub const LEN: usize = 8 + // discriminator
        8 + // total_collected
        1;  // bump
}

