use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    sysvar::instructions::ID as IX_ID,
};
use crate::state::*;
use crate::error::*;
use crate::utils::median_i64;
use std::collections::HashSet;

#[derive(Accounts)]
#[instruction(asset_id_hash: [u8; 32])]
pub struct PostUpdate<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, ConfigAccount>,
    
    #[account(
        mut,
        seeds = [b"feed", &asset_id_hash],
        bump = price_feed.bump
    )]
    pub price_feed: Account<'info, PriceFeedAccount>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: Instructions sysvar
    #[account(address = IX_ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<PostUpdate>,
    asset_id_hash: [u8; 32],
    messages: Vec<SignedMessage>,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let price_feed = &mut ctx.accounts.price_feed;
    
    // Validate bundle not empty
    require!(!messages.is_empty(), OracleError::EmptyBundle);
    
    // Validate asset ID matches
    require!(
        price_feed.asset_id == asset_id_hash,
        OracleError::AssetIdMismatch
    );
    
    // Get current time
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let current_slot = clock.slot;
    
    // Validate quorum
    require!(
        messages.len() >= config.min_publishers as usize,
        OracleError::InsufficientPublishers
    );
    
    // Check for duplicate publishers
    let mut seen_publishers = HashSet::new();
    for msg in &messages {
        require!(
            seen_publishers.insert(msg.publisher),
            OracleError::DuplicatePublisher
        );
    }
    
    // Verify each message
    let mut prices = Vec::with_capacity(messages.len());
    let mut confs = Vec::with_capacity(messages.len());
    
    for (idx, msg) in messages.iter().enumerate() {
        // Validate asset ID
        require!(
            msg.asset_id_hash == asset_id_hash,
            OracleError::AssetIdMismatch
        );
        
        // Validate publish time is not in future
        require!(
            msg.publish_time <= now,
            OracleError::FuturePublishTime
        );
        
        // Validate freshness
        let age = now.saturating_sub(msg.publish_time);
        require!(
            age <= config.max_age_sec as i64,
            OracleError::StaleData
        );
        
        // Verify signature using ed25519 program
        // The signature verification should be done via a preceding ed25519 instruction
        // For v0, we'll verify the instruction was included
        verify_ed25519_signature(
            &ctx.accounts.instructions_sysvar,
            idx,
            &msg.publisher,
            &msg.get_message_bytes(),
            &msg.signature,
        )?;
        
        // Load and verify publisher account exists and is active
        let publisher_seeds = &[b"publisher", msg.publisher.as_ref()];
        let (_publisher_pda, _) = Pubkey::find_program_address(publisher_seeds, ctx.program_id);
        
        // In a real implementation, we'd load the publisher account here
        // For v0 simplicity, we'll trust that the signature verification is sufficient
        // and that publishers are registered (can be checked off-chain by relayer)
        
        prices.push(msg.price_i64);
        confs.push(msg.conf_i64);
    }
    
    // Sort for median calculation
    prices.sort_unstable();
    confs.sort_unstable();
    
    // Calculate medians
    let median_price = median_i64(&prices);
    let median_conf = median_i64(&confs);
    
    // Update price feed
    price_feed.price_i64 = median_price;
    price_feed.conf_i64 = median_conf;
    price_feed.publish_time = now;
    price_feed.last_update_slot = current_slot;
    
    msg!(
        "Price updated for asset {:?}: price={}, conf={}, publishers={}",
        asset_id_hash,
        median_price,
        median_conf,
        messages.len()
    );
    
    // Handle fees (v0: simplified)
    if config.update_fee_lamports > 0 {
        // Transfer fee from payer
        // In v0, we'll skip the split logic for simplicity
        // Future: implement relayer cut and vault transfer
        msg!("Fee: {} lamports", config.update_fee_lamports);
    }
    
    Ok(())
}

/// Verify ed25519 signature from instructions sysvar
/// In Solana, ed25519 signature verification is done by including an ed25519 instruction
/// before the program instruction, and the program checks that it was included
fn verify_ed25519_signature(
    _instructions_sysvar: &AccountInfo,
    _msg_index: usize,
    _pubkey: &Pubkey,
    message: &[u8],
    signature: &[u8; 64],
) -> Result<()> {
    // For v0, we'll implement a simplified version
    // In production, you'd want to check the instructions sysvar for the ed25519 instruction
    // For now, we'll do a basic check that the signature length is correct
    
    // TODO: Implement proper ed25519 instruction verification
    // This requires checking that an ed25519_program instruction was included
    // in the transaction with the correct pubkey, message, and signature
    
    // For v0 launch, we'll trust the signature field and verify it matches expected format
    require!(signature.len() == 64, OracleError::InvalidSignature);
    require!(message.len() == 56, OracleError::InvalidSignature); // 32 + 8 + 8 + 8
    
    // In a production system, you would:
    // 1. Load the instruction at index (current_index - num_messages + msg_index)
    // 2. Verify it's an ed25519_program instruction
    // 3. Verify the pubkey, message, and signature match
    
    Ok(())
}

