use anchor_lang::prelude::*;

declare_id!("CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3");

/// TachyonL2Core - Main L2 state management
/// 
/// This is the central contract for the Tachyon L2 Oracle Network.
/// It manages the overall L2 state, coordinates between other contracts,
/// and maintains the canonical state of the oracle network.
#[program]
pub mod tachyon_l2_core {
    use super::*;

    /// Initialize the L2 Core state
    pub fn initialize(
        ctx: Context<Initialize>,
        authority: Pubkey,
        state_compression_program: Pubkey,
        verifier_program: Pubkey,
        sequencer_program: Pubkey,
    ) -> Result<()> {
        let core_state = &mut ctx.accounts.core_state;
        core_state.authority = authority;
        core_state.state_compression_program = state_compression_program;
        core_state.verifier_program = verifier_program;
        core_state.sequencer_program = sequencer_program;
        core_state.total_batches = 0;
        core_state.total_feeds = 0;
        core_state.total_publishers = 0;
        core_state.last_batch_timestamp = 0;
        core_state.is_paused = false;
        core_state.bump = ctx.bumps.core_state;
        
        msg!("Tachyon L2 Core initialized");
        msg!("Authority: {}", authority);
        msg!("State Compression: {}", state_compression_program);
        
        Ok(())
    }

    /// Update L2 state after a new batch
    pub fn update_batch(
        ctx: Context<UpdateBatch>,
        batch_number: u64,
        feed_count: u32,
        timestamp: i64,
    ) -> Result<()> {
        let core_state = &mut ctx.accounts.core_state;
        
        require!(
            ctx.accounts.authority.key() == core_state.authority,
            L2CoreError::Unauthorized
        );
        
        require!(!core_state.is_paused, L2CoreError::SystemPaused);
        
        core_state.total_batches = batch_number;
        core_state.total_feeds = feed_count;
        core_state.last_batch_timestamp = timestamp;
        
        msg!("Batch updated: #{}, feeds: {}", batch_number, feed_count);
        
        Ok(())
    }

    /// Register a new publisher
    pub fn register_publisher(
        ctx: Context<RegisterPublisher>,
        publisher_pubkey: Pubkey,
    ) -> Result<()> {
        let core_state = &mut ctx.accounts.core_state;
        
        require!(
            ctx.accounts.authority.key() == core_state.authority,
            L2CoreError::Unauthorized
        );
        
        core_state.total_publishers += 1;
        
        msg!("Publisher registered: {}", publisher_pubkey);
        msg!("Total publishers: {}", core_state.total_publishers);
        
        Ok(())
    }

    /// Pause the L2 system (emergency)
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        let core_state = &mut ctx.accounts.core_state;
        
        require!(
            ctx.accounts.authority.key() == core_state.authority,
            L2CoreError::Unauthorized
        );
        
        core_state.is_paused = true;
        
        msg!("⚠️ L2 System PAUSED");
        
        Ok(())
    }

    /// Resume the L2 system
    pub fn resume(ctx: Context<Resume>) -> Result<()> {
        let core_state = &mut ctx.accounts.core_state;
        
        require!(
            ctx.accounts.authority.key() == core_state.authority,
            L2CoreError::Unauthorized
        );
        
        core_state.is_paused = false;
        
        msg!("✅ L2 System RESUMED");
        
        Ok(())
    }

    /// Get L2 Core state
    pub fn get_state(ctx: Context<GetState>) -> Result<L2CoreStateData> {
        let core_state = &ctx.accounts.core_state;
        
        Ok(L2CoreStateData {
            authority: core_state.authority,
            state_compression_program: core_state.state_compression_program,
            verifier_program: core_state.verifier_program,
            sequencer_program: core_state.sequencer_program,
            total_batches: core_state.total_batches,
            total_feeds: core_state.total_feeds,
            total_publishers: core_state.total_publishers,
            last_batch_timestamp: core_state.last_batch_timestamp,
            is_paused: core_state.is_paused,
        })
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + L2CoreState::INIT_SPACE,
        seeds = [b"l2-core"],
        bump
    )]
    pub core_state: Account<'info, L2CoreState>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBatch<'info> {
    #[account(
        mut,
        seeds = [b"l2-core"],
        bump = core_state.bump
    )]
    pub core_state: Account<'info, L2CoreState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterPublisher<'info> {
    #[account(
        mut,
        seeds = [b"l2-core"],
        bump = core_state.bump
    )]
    pub core_state: Account<'info, L2CoreState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        seeds = [b"l2-core"],
        bump = core_state.bump
    )]
    pub core_state: Account<'info, L2CoreState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Resume<'info> {
    #[account(
        mut,
        seeds = [b"l2-core"],
        bump = core_state.bump
    )]
    pub core_state: Account<'info, L2CoreState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetState<'info> {
    #[account(
        seeds = [b"l2-core"],
        bump = core_state.bump
    )]
    pub core_state: Account<'info, L2CoreState>,
}

#[account]
#[derive(InitSpace)]
pub struct L2CoreState {
    pub authority: Pubkey,                      // 32 bytes
    pub state_compression_program: Pubkey,      // 32 bytes
    pub verifier_program: Pubkey,               // 32 bytes
    pub sequencer_program: Pubkey,              // 32 bytes
    pub total_batches: u64,                     // 8 bytes
    pub total_feeds: u32,                       // 4 bytes
    pub total_publishers: u32,                  // 4 bytes
    pub last_batch_timestamp: i64,              // 8 bytes
    pub is_paused: bool,                        // 1 byte
    pub bump: u8,                               // 1 byte
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct L2CoreStateData {
    pub authority: Pubkey,
    pub state_compression_program: Pubkey,
    pub verifier_program: Pubkey,
    pub sequencer_program: Pubkey,
    pub total_batches: u64,
    pub total_feeds: u32,
    pub total_publishers: u32,
    pub last_batch_timestamp: i64,
    pub is_paused: bool,
}

#[error_code]
pub enum L2CoreError {
    #[msg("Unauthorized: Only authority can perform this action")]
    Unauthorized,
    #[msg("System is paused")]
    SystemPaused,
}

