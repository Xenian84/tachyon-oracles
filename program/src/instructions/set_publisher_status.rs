use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

#[derive(Accounts)]
pub struct SetPublisherStatus<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ OracleError::Unauthorized
    )]
    pub config: Account<'info, ConfigAccount>,
    
    #[account(
        mut,
        seeds = [b"publisher", publisher_account.publisher.as_ref()],
        bump = publisher_account.bump
    )]
    pub publisher_account: Account<'info, PublisherAccount>,
    
    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<SetPublisherStatus>, is_active: bool) -> Result<()> {
    let publisher_account = &mut ctx.accounts.publisher_account;
    publisher_account.is_active = is_active;
    
    msg!(
        "Publisher {} status set to: {}",
        publisher_account.publisher,
        is_active
    );
    
    Ok(())
}

