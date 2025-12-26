use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct RegisterPublisher<'info> {
    #[account(
        init,
        payer = publisher,
        space = PublisherAccount::LEN,
        seeds = [b"publisher", publisher.key().as_ref()],
        bump
    )]
    pub publisher_account: Account<'info, PublisherAccount>,
    
    #[account(mut)]
    pub publisher: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterPublisher>) -> Result<()> {
    let publisher_account = &mut ctx.accounts.publisher_account;
    publisher_account.publisher = ctx.accounts.publisher.key();
    publisher_account.staked_amount = 0; // v0: not enforced
    publisher_account.is_active = true;
    publisher_account.bump = ctx.bumps.publisher_account;
    
    msg!("Publisher registered: {}", publisher_account.publisher);
    
    Ok(())
}

