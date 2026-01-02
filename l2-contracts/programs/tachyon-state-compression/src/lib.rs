use anchor_lang::prelude::*;
use solana_program::keccak;

declare_id!("L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx");

// TachyonSequencer program ID for cross-program checks
const SEQUENCER_PROGRAM_ID: Pubkey = solana_program::pubkey!("SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M");

#[program]
pub mod tachyon_state_compression {
    use super::*;

    /// Initialize the L2 state account
    pub fn initialize(ctx: Context<Initialize>, authority: Pubkey) -> Result<()> {
        let l2_state = &mut ctx.accounts.l2_state;
        l2_state.authority = authority;
        l2_state.current_root = [0u8; 32];
        l2_state.batch_number = 0;
        l2_state.feed_count = 0;
        l2_state.last_update = Clock::get()?.unix_timestamp;
        l2_state.bump = ctx.bumps.l2_state;
        
        msg!("L2 State initialized with authority: {}", authority);
        Ok(())
    }

    /// Transfer authority to a new wallet
    pub fn transfer_authority(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
        let l2_state = &mut ctx.accounts.l2_state;
        let old_authority = l2_state.authority;
        l2_state.authority = new_authority;
        
        msg!("Authority transferred from {} to {}", old_authority, new_authority);
        Ok(())
    }

    /// Submit a new Merkle root (with sequencer authorization)
    pub fn submit_root(
        ctx: Context<SubmitRoot>,
        root: [u8; 32],
        feed_count: u32,
        timestamp: i64,
    ) -> Result<()> {
        let l2_state = &mut ctx.accounts.l2_state;
        
        // Check if submitter is authorized sequencer
        // Option 1: Check if it's the authority (for backwards compatibility)
        // Option 2: Check if registered in TachyonSequencer (for multi-validator)
        let is_authority = ctx.accounts.authority.key() == l2_state.authority;
        let is_sequencer = ctx.accounts.sequencer_info.is_some();
        
        require!(
            is_authority || is_sequencer,
            L2Error::Unauthorized
        );
        
        // If sequencer info provided, verify it's active
        if let Some(sequencer_info) = &ctx.accounts.sequencer_info {
            require!(
                sequencer_info.is_active,
                L2Error::SequencerNotActive
            );
            require!(
                sequencer_info.sequencer == ctx.accounts.authority.key(),
                L2Error::InvalidSequencer
            );
        }
        
        // Update state
        l2_state.current_root = root;
        l2_state.batch_number += 1;
        l2_state.feed_count = feed_count;
        l2_state.last_update = timestamp;
        
        msg!(
            "New root submitted: batch={}, feeds={}, root={:?}, submitter={}",
            l2_state.batch_number,
            feed_count,
            &root[..8],
            ctx.accounts.authority.key()
        );
        
        Ok(())
    }

    /// Submit root with consensus votes (2/3 stake verification)
    /// Submit root with consensus votes (2/3 stake verification)
    /// Note: In production, this would parse governance_state account data
    /// For now, simplified to accept total_stake as parameter
    pub fn submit_root_with_consensus(
        ctx: Context<SubmitRootWithConsensus>,
        root: [u8; 32],
        feed_count: u32,
        timestamp: i64,
        total_stake: u64,
        votes: Vec<ConsensusVote>,
    ) -> Result<()> {
        let l2_state = &mut ctx.accounts.l2_state;
        
        // Verify we have enough votes (2/3 of total stake)
        let mut root_votes: std::collections::HashMap<[u8; 32], u64> = std::collections::HashMap::new();
        
        for vote in &votes {
            // TODO: Verify signature in production
            // For now, trust the votes
            *root_votes.entry(vote.root).or_insert(0) += vote.stake;
        }
        
        // Find the root with most stake
        let mut max_stake = 0u64;
        let mut consensus_root = [0u8; 32];
        for (voted_root, stake) in root_votes.iter() {
            if *stake > max_stake {
                max_stake = *stake;
                consensus_root = *voted_root;
            }
        }
        
        // Verify 2/3 threshold
        let quorum_threshold = (total_stake * 2) / 3;
        require!(
            max_stake >= quorum_threshold,
            L2Error::InsufficientConsensus
        );
        
        // Verify the consensus root matches submitted root
        require!(
            consensus_root == root,
            L2Error::RootMismatch
        );
        
        // Update state
        l2_state.current_root = root;
        l2_state.batch_number += 1;
        l2_state.feed_count = feed_count;
        l2_state.last_update = timestamp;
        
        msg!(
            "✅ Consensus reached: {}/{} stake agrees on root",
            max_stake,
            total_stake
        );
        msg!(
            "New root submitted: batch={}, feeds={}, root={:?}",
            l2_state.batch_number,
            feed_count,
            &root[..8]
        );
        
        Ok(())
    }

    /// Verify a Merkle proof and return the price
    pub fn verify_proof(
        ctx: Context<VerifyProof>,
        asset_id: [u8; 32],
        price: i64,
        confidence: i64,
        timestamp: i64,
        proof: Vec<[u8; 32]>,
    ) -> Result<PriceData> {
        let l2_state = &ctx.accounts.l2_state;
        
        // Serialize the price feed (same as L2 aggregator)
        let mut leaf_data = Vec::with_capacity(56);
        leaf_data.extend_from_slice(&asset_id);
        leaf_data.extend_from_slice(&price.to_le_bytes());
        leaf_data.extend_from_slice(&confidence.to_le_bytes());
        leaf_data.extend_from_slice(&timestamp.to_le_bytes());
        
        // Hash the leaf using keccak256
        let mut current_hash = keccak::hash(&leaf_data).to_bytes();
        
        // Verify proof
        for proof_element in proof.iter() {
            current_hash = if current_hash < *proof_element {
                keccak::hash(&[&current_hash[..], &proof_element[..]].concat()).to_bytes()
            } else {
                keccak::hash(&[&proof_element[..], &current_hash[..]].concat()).to_bytes()
            };
        }
        
        // Check if computed root matches stored root
        require!(
            current_hash == l2_state.current_root,
            L2Error::InvalidProof
        );
        
        msg!(
            "Proof verified for asset {:?}: price={}, conf={}",
            &asset_id[..8],
            price,
            confidence
        );
        
        Ok(PriceData {
            asset_id,
            price,
            confidence,
            timestamp,
            batch_number: l2_state.batch_number,
        })
    }

    /// Get the current L2 state
    pub fn get_state(ctx: Context<GetState>) -> Result<L2StateData> {
        let l2_state = &ctx.accounts.l2_state;
        
        Ok(L2StateData {
            authority: l2_state.authority,
            current_root: l2_state.current_root,
            batch_number: l2_state.batch_number,
            feed_count: l2_state.feed_count,
            last_update: l2_state.last_update,
        })
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + L2State::INIT_SPACE,
        seeds = [b"l2-state"],
        bump
    )]
    pub l2_state: Account<'info, L2State>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(
        mut,
        seeds = [b"l2-state"],
        bump = l2_state.bump,
        has_one = authority
    )]
    pub l2_state: Account<'info, L2State>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitRoot<'info> {
    #[account(
        mut,
        seeds = [b"l2-state"],
        bump = l2_state.bump
    )]
    pub l2_state: Account<'info, L2State>,
    
    pub authority: Signer<'info>,
    
    /// Optional: Sequencer info account for multi-validator authorization
    /// If provided, will check if submitter is registered sequencer
    #[account(
        seeds = [b"sequencer-info", authority.key().as_ref()],
        bump,
        seeds::program = SEQUENCER_PROGRAM_ID,
    )]
    pub sequencer_info: Option<Account<'info, SequencerInfo>>,
}

/// Sequencer info from TachyonSequencer program (for CPI verification)
#[account]
pub struct SequencerInfo {
    pub sequencer: Pubkey,
    pub stake_amount: u64,
    pub is_active: bool,
    pub total_submissions: u64,
    pub last_submission: i64,
}

#[derive(Accounts)]
pub struct VerifyProof<'info> {
    #[account(
        seeds = [b"l2-state"],
        bump = l2_state.bump
    )]
    pub l2_state: Account<'info, L2State>,
}

#[derive(Accounts)]
pub struct GetState<'info> {
    #[account(
        seeds = [b"l2-state"],
        bump = l2_state.bump
    )]
    pub l2_state: Account<'info, L2State>,
}

#[derive(Accounts)]
pub struct SubmitRootWithConsensus<'info> {
    #[account(
        mut,
        seeds = [b"l2-state"],
        bump = l2_state.bump
    )]
    pub l2_state: Account<'info, L2State>,
    
    /// CHECK: TachyonGovernance state for total stake verification (read-only)
    pub governance_state: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
    
    /// CHECK: TachyonGovernance program
    pub governance_program: AccountInfo<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct L2State {
    pub authority: Pubkey,          // 32 bytes
    pub current_root: [u8; 32],     // 32 bytes
    pub batch_number: u64,          // 8 bytes
    pub feed_count: u32,            // 4 bytes
    pub last_update: i64,           // 8 bytes
    pub bump: u8,                   // 1 byte
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    pub asset_id: [u8; 32],
    pub price: i64,
    pub confidence: i64,
    pub timestamp: i64,
    pub batch_number: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct L2StateData {
    pub authority: Pubkey,
    pub current_root: [u8; 32],
    pub batch_number: u64,
    pub feed_count: u32,
    pub last_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConsensusVote {
    pub validator: Pubkey,
    pub root: [u8; 32],
    pub stake: u64,
    pub signature: [u8; 64],
}

#[error_code]
pub enum L2Error {
    #[msg("Unauthorized: Only authority or registered sequencer can submit roots")]
    Unauthorized,
    #[msg("Invalid Merkle proof")]
    InvalidProof,
    #[msg("Sequencer is not active")]
    SequencerNotActive,
    #[msg("Invalid sequencer account")]
    InvalidSequencer,
    #[msg("Insufficient consensus: need 2/3 stake agreement")]
    InsufficientConsensus,
    #[msg("Root mismatch: consensus root doesn't match submitted root")]
    RootMismatch,
}
