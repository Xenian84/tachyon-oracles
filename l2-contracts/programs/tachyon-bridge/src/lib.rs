use anchor_lang::prelude::*;

declare_id!("BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW");

/// TachyonBridge - Cross-chain oracle data bridge
/// 
/// This contract enables cross-chain oracle data transfer,
/// allowing other blockchains to consume Tachyon price feeds
/// through Wormhole or other bridge protocols.
#[program]
pub mod tachyon_bridge {
    use super::*;

    /// Initialize the bridge
    pub fn initialize(
        ctx: Context<Initialize>,
        authority: Pubkey,
        supported_chains: Vec<u16>,
    ) -> Result<()> {
        let bridge_state = &mut ctx.accounts.bridge_state;
        bridge_state.authority = authority;
        bridge_state.total_messages_sent = 0;
        bridge_state.total_messages_received = 0;
        bridge_state.is_active = true;
        bridge_state.bump = ctx.bumps.bridge_state;
        
        msg!("Tachyon Bridge initialized");
        msg!("Supported chains: {:?}", supported_chains);
        
        Ok(())
    }

    /// Send price data to another chain
    pub fn send_cross_chain(
        ctx: Context<SendCrossChain>,
        target_chain: u16,
        asset_id: [u8; 32],
        price: i64,
        confidence: i64,
        timestamp: i64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let bridge_state = &mut ctx.accounts.bridge_state;
        
        require!(bridge_state.is_active, BridgeError::BridgeInactive);
        
        // Create cross-chain message
        let message = CrossChainMessage {
            source_chain: 1, // X1 chain ID
            target_chain,
            asset_id,
            price,
            confidence,
            timestamp,
            merkle_proof: merkle_proof.clone(),
            nonce: bridge_state.total_messages_sent,
        };
        
        bridge_state.total_messages_sent += 1;
        
        msg!("📤 Cross-chain message sent");
        msg!("Target chain: {}, Asset: {:?}", target_chain, &asset_id[..8]);
        msg!("Price: {}, Nonce: {}", price, message.nonce);
        
        // TODO: Integrate with Wormhole or other bridge
        // wormhole::post_message(ctx, message)?;
        
        Ok(())
    }

    /// Receive price data from another chain
    pub fn receive_cross_chain(
        ctx: Context<ReceiveCrossChain>,
        message: CrossChainMessage,
        signatures: Vec<[u8; 65]>,
    ) -> Result<()> {
        let bridge_state = &mut ctx.accounts.bridge_state;
        
        require!(bridge_state.is_active, BridgeError::BridgeInactive);
        
        // Verify signatures (multi-sig validation)
        require!(
            signatures.len() >= 2, // Minimum 2 signatures
            BridgeError::InsufficientSignatures
        );
        
        // TODO: Verify signatures against guardian set
        // verify_guardian_signatures(&message, &signatures)?;
        
        bridge_state.total_messages_received += 1;
        
        msg!("📥 Cross-chain message received");
        msg!("Source chain: {}, Asset: {:?}", message.source_chain, &message.asset_id[..8]);
        msg!("Price: {}, Nonce: {}", message.price, message.nonce);
        
        Ok(())
    }

    /// Register a new supported chain
    pub fn add_chain(
        ctx: Context<AddChain>,
        chain_id: u16,
        chain_name: String,
    ) -> Result<()> {
        let bridge_state = &ctx.accounts.bridge_state;
        
        require!(
            ctx.accounts.authority.key() == bridge_state.authority,
            BridgeError::Unauthorized
        );
        
        msg!("New chain registered: {} (ID: {})", chain_name, chain_id);
        
        Ok(())
    }

    /// Pause/unpause the bridge
    pub fn set_active(
        ctx: Context<SetActive>,
        is_active: bool,
    ) -> Result<()> {
        let bridge_state = &mut ctx.accounts.bridge_state;
        
        require!(
            ctx.accounts.authority.key() == bridge_state.authority,
            BridgeError::Unauthorized
        );
        
        bridge_state.is_active = is_active;
        
        msg!(
            "Bridge {}",
            if is_active { "ACTIVATED" } else { "PAUSED" }
        );
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + BridgeState::INIT_SPACE,
        seeds = [b"bridge"],
        bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendCrossChain<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
    
    pub sender: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReceiveCrossChain<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
}

#[derive(Accounts)]
pub struct AddChain<'info> {
    #[account(
        seeds = [b"bridge"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetActive<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
    
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct BridgeState {
    pub authority: Pubkey,              // 32 bytes
    pub total_messages_sent: u64,       // 8 bytes
    pub total_messages_received: u64,   // 8 bytes
    pub is_active: bool,                // 1 byte
    pub bump: u8,                       // 1 byte
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrossChainMessage {
    pub source_chain: u16,
    pub target_chain: u16,
    pub asset_id: [u8; 32],
    pub price: i64,
    pub confidence: i64,
    pub timestamp: i64,
    pub merkle_proof: Vec<[u8; 32]>,
    pub nonce: u64,
}

#[error_code]
pub enum BridgeError {
    #[msg("Unauthorized: Only authority can perform this action")]
    Unauthorized,
    #[msg("Bridge is not active")]
    BridgeInactive,
    #[msg("Insufficient signatures for cross-chain message")]
    InsufficientSignatures,
}

