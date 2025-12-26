use anchor_lang::prelude::*;

// Tachyon Oracles Program ID
declare_id!("TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1");

pub mod state;
pub mod instructions;
pub mod error;
pub mod utils;

use instructions::*;
use state::SignedMessage;

#[program]
pub mod tachyon_oracles {
    use super::*;

    /// Initialize the oracle configuration
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        update_fee_lamports: u64,
        relayer_cut_bps: u16,
        min_publishers: u8,
        max_age_sec: u32,
    ) -> Result<()> {
        instructions::initialize_config::handler(
            ctx,
            update_fee_lamports,
            relayer_cut_bps,
            min_publishers,
            max_age_sec,
        )
    }

    /// Add a new asset to track
    pub fn add_asset(ctx: Context<AddAsset>, asset_id_string: String) -> Result<()> {
        instructions::add_asset::handler(ctx, asset_id_string)
    }

    /// Register a new publisher
    pub fn register_publisher(ctx: Context<RegisterPublisher>) -> Result<()> {
        instructions::register_publisher::handler(ctx)
    }

    /// Set publisher active status (admin only)
    pub fn set_publisher_status(
        ctx: Context<SetPublisherStatus>,
        is_active: bool,
    ) -> Result<()> {
        instructions::set_publisher_status::handler(ctx, is_active)
    }

    /// Post a price update with signed messages from publishers
    pub fn post_update(
        ctx: Context<PostUpdate>,
        asset_id_hash: [u8; 32],
        messages: Vec<SignedMessage>,
    ) -> Result<()> {
        instructions::post_update::handler(ctx, asset_id_hash, messages)
    }
}
