use anchor_lang::prelude::*;

declare_id!("SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M");

/// TachyonSequencer - Batch submission authority
/// 
/// This contract manages the authority and permissions for submitting
/// batches to the L2. It implements a multi-sig or stake-weighted
/// sequencer model for decentralization.
#[program]
pub mod tachyon_sequencer {
    use super::*;

    /// Initialize the sequencer state
    pub fn initialize(
        ctx: Context<Initialize>,
        authority: Pubkey,
        min_stake: u64,
    ) -> Result<()> {
        let sequencer_state = &mut ctx.accounts.sequencer_state;
        sequencer_state.authority = authority;
        sequencer_state.min_stake = min_stake;
        sequencer_state.active_sequencers = 0;
        sequencer_state.total_batches_submitted = 0;
        sequencer_state.is_permissioned = true;
        sequencer_state.bump = ctx.bumps.sequencer_state;
        
        msg!("Tachyon Sequencer initialized");
        msg!("Min stake: {} TACH", min_stake);
        
        Ok(())
    }

    /// Transfer authority to a new wallet
    pub fn transfer_authority(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
        let sequencer_state = &mut ctx.accounts.sequencer_state;
        let old_authority = sequencer_state.authority;
        sequencer_state.authority = new_authority;
        
        msg!("Sequencer authority transferred from {} to {}", old_authority, new_authority);
        Ok(())
    }

    /// Register a new sequencer (requires stake)
    pub fn register_sequencer(
        ctx: Context<RegisterSequencer>,
        sequencer_pubkey: Pubkey,
        stake_amount: u64,
    ) -> Result<()> {
        let sequencer_state = &mut ctx.accounts.sequencer_state;
        
        require!(
            ctx.accounts.authority.key() == sequencer_state.authority,
            SequencerError::Unauthorized
        );
        
        require!(
            stake_amount >= sequencer_state.min_stake,
            SequencerError::InsufficientStake
        );
        
        let sequencer_info = &mut ctx.accounts.sequencer_info;
        sequencer_info.pubkey = sequencer_pubkey;
        sequencer_info.stake_amount = stake_amount;
        sequencer_info.batches_submitted = 0;
        sequencer_info.is_active = true;
        sequencer_info.registered_at = Clock::get()?.unix_timestamp;
        sequencer_info.bump = ctx.bumps.sequencer_info;
        
        sequencer_state.active_sequencers += 1;
        
        msg!("Sequencer registered: {}", sequencer_pubkey);
        msg!("Stake: {} TACH", stake_amount);
        
        Ok(())
    }

    /// Submit a batch (only authorized sequencers)
    pub fn submit_batch(
        ctx: Context<SubmitBatch>,
        batch_number: u64,
        merkle_root: [u8; 32],
        feed_count: u32,
    ) -> Result<()> {
        let sequencer_info = &mut ctx.accounts.sequencer_info;
        let sequencer_state = &mut ctx.accounts.sequencer_state;
        
        require!(sequencer_info.is_active, SequencerError::SequencerInactive);
        
        sequencer_info.batches_submitted += 1;
        sequencer_state.total_batches_submitted += 1;
        
        msg!(
            "Batch #{} submitted by {}",
            batch_number,
            sequencer_info.pubkey
        );
        msg!("Root: {:?}, Feeds: {}", &merkle_root[..8], feed_count);
        
        Ok(())
    }

    /// Slash a sequencer for misbehavior
    pub fn slash_sequencer(
        ctx: Context<SlashSequencer>,
        slash_amount: u64,
        reason: String,
    ) -> Result<()> {
        let sequencer_state = &ctx.accounts.sequencer_state;
        
        require!(
            ctx.accounts.authority.key() == sequencer_state.authority,
            SequencerError::Unauthorized
        );
        
        let sequencer_info = &mut ctx.accounts.sequencer_info;
        
        require!(
            sequencer_info.stake_amount >= slash_amount,
            SequencerError::InsufficientStake
        );
        
        sequencer_info.stake_amount -= slash_amount;
        
        if sequencer_info.stake_amount < sequencer_state.min_stake {
            sequencer_info.is_active = false;
        }
        
        msg!("⚠️ Sequencer slashed: {}", sequencer_info.pubkey);
        msg!("Amount: {} TACH, Reason: {}", slash_amount, reason);
        
        Ok(())
    }

    /// Toggle permissioned mode
    pub fn set_permissioned(
        ctx: Context<SetPermissioned>,
        is_permissioned: bool,
    ) -> Result<()> {
        let sequencer_state = &mut ctx.accounts.sequencer_state;
        
        require!(
            ctx.accounts.authority.key() == sequencer_state.authority,
            SequencerError::Unauthorized
        );
        
        sequencer_state.is_permissioned = is_permissioned;
        
        msg!(
            "Sequencer mode: {}",
            if is_permissioned { "Permissioned" } else { "Permissionless" }
        );
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + SequencerState::INIT_SPACE,
        seeds = [b"sequencer"],
        bump
    )]
    pub sequencer_state: Account<'info, SequencerState>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(
        mut,
        seeds = [b"sequencer"],
        bump = sequencer_state.bump,
        has_one = authority
    )]
    pub sequencer_state: Account<'info, SequencerState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(sequencer_pubkey: Pubkey)]
pub struct RegisterSequencer<'info> {
    #[account(
        mut,
        seeds = [b"sequencer"],
        bump = sequencer_state.bump
    )]
    pub sequencer_state: Account<'info, SequencerState>,
    
    #[account(
        init,
        payer = payer,
        space = 8 + SequencerInfo::INIT_SPACE,
        seeds = [b"sequencer-info", sequencer_pubkey.as_ref()],
        bump
    )]
    pub sequencer_info: Account<'info, SequencerInfo>,
    
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitBatch<'info> {
    #[account(
        mut,
        seeds = [b"sequencer"],
        bump = sequencer_state.bump
    )]
    pub sequencer_state: Account<'info, SequencerState>,
    
    #[account(
        mut,
        seeds = [b"sequencer-info", sequencer.key().as_ref()],
        bump = sequencer_info.bump
    )]
    pub sequencer_info: Account<'info, SequencerInfo>,
    
    pub sequencer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SlashSequencer<'info> {
    #[account(
        seeds = [b"sequencer"],
        bump = sequencer_state.bump
    )]
    pub sequencer_state: Account<'info, SequencerState>,
    
    #[account(
        mut,
        seeds = [b"sequencer-info", sequencer_info.pubkey.as_ref()],
        bump = sequencer_info.bump
    )]
    pub sequencer_info: Account<'info, SequencerInfo>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPermissioned<'info> {
    #[account(
        mut,
        seeds = [b"sequencer"],
        bump = sequencer_state.bump
    )]
    pub sequencer_state: Account<'info, SequencerState>,
    
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct SequencerState {
    pub authority: Pubkey,              // 32 bytes
    pub min_stake: u64,                 // 8 bytes
    pub active_sequencers: u32,         // 4 bytes
    pub total_batches_submitted: u64,   // 8 bytes
    pub is_permissioned: bool,          // 1 byte
    pub bump: u8,                       // 1 byte
}

#[account]
#[derive(InitSpace)]
pub struct SequencerInfo {
    pub pubkey: Pubkey,                 // 32 bytes
    pub stake_amount: u64,              // 8 bytes
    pub batches_submitted: u64,         // 8 bytes
    pub is_active: bool,                // 1 byte
    pub registered_at: i64,             // 8 bytes
    pub bump: u8,                       // 1 byte
}

#[error_code]
pub enum SequencerError {
    #[msg("Unauthorized: Only authority can perform this action")]
    Unauthorized,
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    #[msg("Sequencer is not active")]
    SequencerInactive,
}

