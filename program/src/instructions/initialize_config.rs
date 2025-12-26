use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = ConfigAccount::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, ConfigAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeConfig>,
    update_fee_lamports: u64,
    relayer_cut_bps: u16,
    min_publishers: u8,
    max_age_sec: u32,
) -> Result<()> {
    require!(relayer_cut_bps <= 10000, OracleError::InvalidRelayerCut);
    
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.update_fee_lamports = update_fee_lamports;
    config.relayer_cut_bps = relayer_cut_bps;
    config.min_publishers = min_publishers;
    config.max_age_sec = max_age_sec;
    config.asset_count = 0;
    config.bump = ctx.bumps.config;
    
    msg!("Oracle config initialized by admin: {}", config.admin);
    msg!("Min publishers: {}, Max age: {}s", min_publishers, max_age_sec);
    
    Ok(())
}

