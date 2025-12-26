use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::utils::hash_asset_id;

#[derive(Accounts)]
#[instruction(asset_id_string: String)]
pub struct AddAsset<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ OracleError::Unauthorized
    )]
    pub config: Account<'info, ConfigAccount>,
    
    #[account(
        init,
        payer = admin,
        space = PriceFeedAccount::LEN,
        seeds = [b"feed", hash_asset_id(&asset_id_string).as_ref()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeedAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddAsset>, asset_id_string: String) -> Result<()> {
    require!(!asset_id_string.is_empty(), OracleError::InvalidAssetId);
    require!(asset_id_string.len() <= 64, OracleError::InvalidAssetId);
    
    let asset_id_hash = hash_asset_id(&asset_id_string);
    
    let price_feed = &mut ctx.accounts.price_feed;
    price_feed.asset_id = asset_id_hash;
    price_feed.price_i64 = 0;
    price_feed.conf_i64 = 0;
    price_feed.publish_time = 0;
    price_feed.last_update_slot = 0;
    price_feed.bump = ctx.bumps.price_feed;
    
    let config = &mut ctx.accounts.config;
    config.asset_count = config.asset_count
        .checked_add(1)
        .ok_or(OracleError::ArithmeticOverflow)?;
    
    msg!("Asset added: {} (hash: {:?})", asset_id_string, asset_id_hash);
    
    Ok(())
}

