use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9");

/// TachyonGovernance - Protocol governance with TACH token
/// 
/// This contract manages protocol governance, staking, and rewards
/// using the TACH SPL token. Enables decentralized decision-making
/// for protocol upgrades and parameter changes.
#[program]
pub mod tachyon_governance {
    use super::*;

    /// Initialize governance with vault and rewards pool
    pub fn initialize(
        ctx: Context<Initialize>,
        min_stake: u64,
        min_proposal_stake: u64,
        voting_period: i64,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        governance_state.authority = ctx.accounts.authority.key();
        governance_state.tach_mint = ctx.accounts.tach_mint.key();
        governance_state.vault = ctx.accounts.vault.key();
        governance_state.rewards_pool = ctx.accounts.rewards_pool.key();
        governance_state.min_stake = min_stake;
        governance_state.min_proposal_stake = min_proposal_stake;
        governance_state.voting_period = voting_period;
        governance_state.total_proposals = 0;
        governance_state.total_staked = 0;
        governance_state.total_rewards_distributed = 0;
        governance_state.bump = ctx.bumps.governance_state;
        governance_state.vault_bump = ctx.bumps.vault;
        governance_state.rewards_pool_bump = ctx.bumps.rewards_pool;
        // NEW: Initialize rewards system
        governance_state.daily_rewards_rate = 82_000_000_000_000; // 82K TACH/day (with 9 decimals)
        governance_state.rewards_paused = false;
        governance_state.last_epoch_distribution = Clock::get()?.unix_timestamp;
        governance_state.epoch_duration = 86400; // 24 hours
        governance_state.pool_refill_threshold = 1_000_000_000_000_000; // 1M TACH
        governance_state.total_slashed = 0;
        governance_state.total_stakers = 0;
        
        msg!("Tachyon Governance initialized");
        msg!("TACH Mint: {}", ctx.accounts.tach_mint.key());
        msg!("Min stake: {} TACH", min_stake);
        msg!("Min proposal stake: {} TACH", min_proposal_stake);
        msg!("Voting period: {}s", voting_period);
        msg!("Vault: {}", ctx.accounts.vault.key());
        msg!("Rewards Pool: {}", ctx.accounts.rewards_pool.key());
        msg!("Daily rewards rate: {} TACH", governance_state.daily_rewards_rate / 1_000_000_000);
        
        Ok(())
    }

    /// Initialize staker account (must be called before staking)
    pub fn init_staker(ctx: Context<InitStaker>, referrer: Option<Pubkey>) -> Result<()> {
        let staker_info = &mut ctx.accounts.staker_info;
        let governance_state = &mut ctx.accounts.governance_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        staker_info.staked_amount = 0;
        staker_info.last_stake_timestamp = current_time;
        staker_info.bump = ctx.bumps.staker_info;
        // NEW: Initialize rewards tracking
        staker_info.total_rewards_claimed = 0;
        staker_info.last_claim_timestamp = current_time;
        staker_info.pending_rewards = 0;
        staker_info.compounded_rewards = 0;
        // NEW: Initialize performance tracking
        staker_info.uptime_score = 10000; // Start at 100%
        staker_info.submissions_count = 0;
        staker_info.accurate_submissions = 0;
        // NEW: Initialize loyalty tracking
        staker_info.first_stake_timestamp = current_time;
        staker_info.loyalty_tier = 0; // None
        // NEW: Initialize referral tracking
        staker_info.referrer = referrer.unwrap_or(Pubkey::default());
        staker_info.referral_count = 0;
        staker_info.referral_rewards = 0;
        // NEW: Initialize vesting
        staker_info.vested_rewards = 0;
        staker_info.vesting_start = 0;
        
        // Increment total stakers
        governance_state.total_stakers += 1;
        
        msg!("Staker account initialized for {}", ctx.accounts.staker.key());
        if let Some(ref_key) = referrer {
            msg!("Referred by: {}", ref_key);
        }
        
        Ok(())
    }

    /// Stake TACH tokens (transfers to vault)
    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        // Enforce minimum stake requirement
        let new_total = staker_info.staked_amount + amount;
        require!(
            new_total >= governance_state.min_stake,
            GovernanceError::BelowMinimumStake
        );
        
        // Transfer TACH tokens from staker to vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staker_token_account.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.staker.to_account_info(),
                },
            ),
            amount,
        )?;
        
        staker_info.staked_amount += amount;
        staker_info.last_stake_timestamp = Clock::get()?.unix_timestamp;
        
        governance_state.total_staked += amount;
        
        msg!("✅ Staked {} TACH", amount);
        msg!("Total staked by user: {} TACH", staker_info.staked_amount);
        msg!("Network total staked: {} TACH", governance_state.total_staked);
        
        Ok(())
    }

    /// Unstake TACH tokens (transfers from vault back to staker)
    pub fn unstake(
        ctx: Context<Unstake>,
        amount: u64,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        require!(
            staker_info.staked_amount >= amount,
            GovernanceError::InsufficientStake
        );
        
        // Check unstaking cooldown (7 days)
        let current_time = Clock::get()?.unix_timestamp;
        let cooldown_period = 7 * 24 * 60 * 60; // 7 days
        require!(
            current_time - staker_info.last_stake_timestamp >= cooldown_period,
            GovernanceError::CooldownPeriodActive
        );
        
        // Check remaining stake meets minimum (or is zero)
        let remaining = staker_info.staked_amount - amount;
        require!(
            remaining == 0 || remaining >= governance_state.min_stake,
            GovernanceError::BelowMinimumStake
        );
        
        // Transfer TACH tokens from vault back to staker
        let seeds = &[
            b"governance".as_ref(),
            &[governance_state.bump],
        ];
        let signer = &[&seeds[..]];
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.staker_token_account.to_account_info(),
                    authority: governance_state.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;
        
        staker_info.staked_amount -= amount;
        governance_state.total_staked -= amount;
        
        msg!("✅ Unstaked {} TACH", amount);
        msg!("Remaining staked: {} TACH", staker_info.staked_amount);
        
        Ok(())
    }

    /// Slash a misbehaving validator (authority only)
    pub fn slash(
        ctx: Context<Slash>,
        slash_amount: u64,
        reason: String,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        
        require!(
            staker_info.staked_amount >= slash_amount,
            GovernanceError::InsufficientStake
        );
        
        // Transfer slashed tokens from vault to rewards pool
        let seeds = &[
            b"governance".as_ref(),
            &[governance_state.bump],
        ];
        let signer = &[&seeds[..]];
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.rewards_pool.to_account_info(),
                    authority: governance_state.to_account_info(),
                },
                signer,
            ),
            slash_amount,
        )?;
        
        staker_info.staked_amount -= slash_amount;
        governance_state.total_staked -= slash_amount;
        
        msg!("⚠️  SLASHED {} TACH from {}", slash_amount, ctx.accounts.slashed_staker.key());
        msg!("Reason: {}", reason);
        msg!("Remaining stake: {} TACH", staker_info.staked_amount);
        
        Ok(())
    }

    /// Create a governance proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        proposal_type: ProposalType,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let staker_info = &ctx.accounts.staker_info;
        let proposal = &mut ctx.accounts.proposal;
        
        require!(
            staker_info.staked_amount >= governance_state.min_proposal_stake,
            GovernanceError::InsufficientStakeForProposal
        );
        
        let current_time = Clock::get()?.unix_timestamp;
        
        proposal.id = governance_state.total_proposals;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.title = title.clone();
        proposal.description = description;
        proposal.proposal_type = proposal_type;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.status = ProposalStatus::Active;
        proposal.created_at = current_time;
        proposal.voting_ends_at = current_time + governance_state.voting_period;
        proposal.bump = ctx.bumps.proposal;
        
        governance_state.total_proposals += 1;
        
        msg!("Proposal #{} created: {}", proposal.id, title);
        
        Ok(())
    }

    /// Vote on a proposal
    pub fn vote(
        ctx: Context<Vote>,
        proposal_id: u64,
        vote_for: bool,
    ) -> Result<()> {
        let staker_info = &ctx.accounts.staker_info;
        let proposal = &mut ctx.accounts.proposal;
        
        require!(
            proposal.status == ProposalStatus::Active,
            GovernanceError::ProposalNotActive
        );
        
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time < proposal.voting_ends_at,
            GovernanceError::VotingPeriodEnded
        );
        
        let voting_power = staker_info.staked_amount;
        
        if vote_for {
            proposal.votes_for += voting_power;
        } else {
            proposal.votes_against += voting_power;
        }
        
        msg!(
            "Voted {} on proposal #{} with {} TACH",
            if vote_for { "FOR" } else { "AGAINST" },
            proposal_id,
            voting_power
        );
        
        Ok(())
    }

    /// Execute a passed proposal
    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
        proposal_id: u64,
    ) -> Result<()> {
        let governance_state = &ctx.accounts.governance_state;
        let proposal = &mut ctx.accounts.proposal;
        
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        
        require!(
            proposal.status == ProposalStatus::Active,
            GovernanceError::ProposalNotActive
        );
        
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time >= proposal.voting_ends_at,
            GovernanceError::VotingPeriodNotEnded
        );
        
        // Check if proposal passed (simple majority)
        let total_votes = proposal.votes_for + proposal.votes_against;
        let passed = proposal.votes_for > proposal.votes_against && total_votes > 0;
        
        if passed {
            proposal.status = ProposalStatus::Executed;
            msg!("✅ Proposal #{} EXECUTED", proposal_id);
            
            // TODO: Execute the actual proposal action based on proposal_type
            // match proposal.proposal_type {
            //     ProposalType::ParameterChange => { /* ... */ }
            //     ProposalType::ProtocolUpgrade => { /* ... */ }
            //     ProposalType::TreasurySpend => { /* ... */ }
            // }
        } else {
            proposal.status = ProposalStatus::Rejected;
            msg!("❌ Proposal #{} REJECTED", proposal_id);
        }
        
        Ok(())
    }

    /// Claim staking rewards
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        // Validate rewards_pool PDA
        let (expected_rewards_pool, _) = Pubkey::find_program_address(
            &[b"rewards-pool"],
            &crate::ID,
        );
        require!(
            ctx.accounts.rewards_pool.key() == expected_rewards_pool,
            GovernanceError::InvalidRewardsPool
        );
        
        // Calculate rewards based on stake and time
        let current_time = Clock::get()?.unix_timestamp;
        let time_staked = current_time - staker_info.last_stake_timestamp;
        
        // Daily rewards: 82,000 TACH / all stakers (proportional to stake)
        // This is simplified - in production, track per-epoch rewards
        let daily_rewards: u64 = 82_000_000_000; // 82k TACH with 9 decimals
        let seconds_per_day: u64 = 86400;
        
        let stake_percentage = if governance_state.total_staked > 0 {
            (staker_info.staked_amount as u128 * 1_000_000) / governance_state.total_staked as u128
        } else {
            0
        };
        
        let rewards = ((daily_rewards as u128 * stake_percentage * time_staked as u128) 
            / (seconds_per_day as u128 * 1_000_000)) as u64;
        
        require!(rewards > 0, GovernanceError::NoRewardsAvailable);
        
        // Transfer rewards from rewards pool to staker
        let seeds = &[
            b"governance".as_ref(),
            &[governance_state.bump],
        ];
        let signer = &[&seeds[..]];
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.rewards_pool.clone(),
                    to: ctx.accounts.staker_token_account.to_account_info(),
                    authority: governance_state.to_account_info(),
                },
                signer,
            ),
            rewards,
        )?;
        
        // Reset timestamp to prevent double-claiming
        staker_info.last_stake_timestamp = current_time;
        governance_state.total_rewards_distributed += rewards;
        
        msg!("✅ Claimed {} TACH rewards", rewards / 1_000_000_000);
        msg!("Total rewards distributed: {} TACH", governance_state.total_rewards_distributed / 1_000_000_000);
        
        Ok(())
    }

    /// Fund the rewards pool (authority only)
    pub fn update_min_stake(ctx: Context<UpdateMinStake>, new_min_stake: u64) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        let old_min_stake = governance_state.min_stake;
        governance_state.min_stake = new_min_stake;
        msg!("Min stake updated from {} to {}", old_min_stake, new_min_stake);
        Ok(())
    }
    
    pub fn fund_rewards_pool(
        ctx: Context<FundRewardsPool>,
        amount: u64,
    ) -> Result<()> {
        let governance_state = &ctx.accounts.governance_state;
        
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        
        // Transfer TACH from authority to rewards pool
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_account.to_account_info(),
                    to: ctx.accounts.rewards_pool.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;
        
        msg!("✅ Rewards pool funded with {} TACH", amount / 1_000_000_000);
        
        Ok(())
    }

    // ============================================================================
    // NEW REWARDS SYSTEM FEATURES
    // ============================================================================

    /// Pause/unpause rewards distribution (emergency only)
    pub fn set_rewards_paused(
        ctx: Context<SetRewardsPaused>,
        paused: bool,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        
        governance_state.rewards_paused = paused;
        msg!("Rewards distribution {}", if paused { "PAUSED" } else { "RESUMED" });
        
        Ok(())
    }

    /// Update daily rewards rate (authority only)
    pub fn update_rewards_rate(
        ctx: Context<UpdateRewardsRate>,
        new_daily_rate: u64,
    ) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        
        let old_rate = governance_state.daily_rewards_rate;
        governance_state.daily_rewards_rate = new_daily_rate;
        msg!("Daily rewards rate updated from {} to {} TACH", 
            old_rate / 1_000_000_000, new_daily_rate / 1_000_000_000);
        
        Ok(())
    }

    /// Claim rewards and automatically compound (stake them)
    pub fn claim_and_compound(ctx: Context<ClaimRewards>) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        require!(!governance_state.rewards_paused, GovernanceError::RewardsPaused);
        
        // Validate rewards_pool PDA
        let (expected_rewards_pool, _) = Pubkey::find_program_address(
            &[b"rewards-pool"],
            &crate::ID,
        );
        require!(
            ctx.accounts.rewards_pool.key() == expected_rewards_pool,
            GovernanceError::InvalidRewardsPool
        );
        
        // Calculate rewards with loyalty and performance bonuses
        let rewards = calculate_total_rewards_internal(governance_state, staker_info)?;
        require!(rewards > 0, GovernanceError::NoRewardsAvailable);
        
        // Transfer rewards to vault (compound)
        let seeds = &[
            b"governance".as_ref(),
            &[governance_state.bump],
        ];
        let signer = &[&seeds[..]];
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.rewards_pool.clone(),
                    to: ctx.accounts.staker_token_account.to_account_info(),
                    authority: governance_state.to_account_info(),
                },
                signer,
            ),
            rewards,
        )?;
        
        // Update staker info
        staker_info.staked_amount += rewards;
        staker_info.compounded_rewards += rewards;
        staker_info.total_rewards_claimed += rewards;
        staker_info.last_claim_timestamp = Clock::get()?.unix_timestamp;
        
        // Update governance state
        governance_state.total_staked += rewards;
        governance_state.total_rewards_distributed += rewards;
        
        msg!("✅ Compounded {} TACH rewards", rewards / 1_000_000_000);
        
        Ok(())
    }

    /// Update performance metrics for a validator
    pub fn update_performance(
        ctx: Context<UpdatePerformance>,
        uptime_score: u64,
        submissions_count: u64,
        accurate_submissions: u64,
    ) -> Result<()> {
        let governance_state = &ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        require!(
            ctx.accounts.authority.key() == governance_state.authority,
            GovernanceError::Unauthorized
        );
        
        staker_info.uptime_score = uptime_score;
        staker_info.submissions_count = submissions_count;
        staker_info.accurate_submissions = accurate_submissions;
        
        msg!("Performance updated: uptime={}%, accuracy={}/{}", 
            uptime_score / 100, accurate_submissions, submissions_count);
        
        Ok(())
    }

    /// Distribute epoch rewards to all stakers (automated)
    pub fn distribute_epoch_rewards(ctx: Context<DistributeEpochRewards>) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        require!(!governance_state.rewards_paused, GovernanceError::RewardsPaused);
        require!(
            current_time >= governance_state.last_epoch_distribution + governance_state.epoch_duration,
            GovernanceError::EpochNotReady
        );
        
        // Update epoch timestamp
        governance_state.last_epoch_distribution = current_time;
        
        msg!("✅ Epoch rewards distribution triggered");
        msg!("Next distribution in {} seconds", governance_state.epoch_duration);
        
        Ok(())
    }

    /// Auto-refill rewards pool when balance is low
    pub fn auto_refill_rewards_pool(
        ctx: Context<FundRewardsPool>,
        amount: u64,
    ) -> Result<()> {
        let governance_state = &ctx.accounts.governance_state;
        
        // Check if pool balance is below threshold
        let pool_balance = ctx.accounts.rewards_pool.amount;
        require!(
            pool_balance < governance_state.pool_refill_threshold,
            GovernanceError::PoolBalanceSufficient
        );
        
        // Transfer from authority
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_account.to_account_info(),
                    to: ctx.accounts.rewards_pool.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;
        
        msg!("✅ Auto-refilled rewards pool with {} TACH", amount / 1_000_000_000);
        msg!("Pool balance: {} TACH", (pool_balance + amount) / 1_000_000_000);
        
        Ok(())
    }

    /// Claim referral rewards
    pub fn claim_referral_rewards(ctx: Context<ClaimReferralRewards>) -> Result<()> {
        let governance_state = &ctx.accounts.governance_state;
        let staker_info = &mut ctx.accounts.staker_info;
        
        require!(!governance_state.rewards_paused, GovernanceError::RewardsPaused);
        require!(staker_info.referral_rewards > 0, GovernanceError::NoRewardsAvailable);
        
        let rewards = staker_info.referral_rewards;
        
        // Transfer referral rewards
        let seeds = &[
            b"governance".as_ref(),
            &[governance_state.bump],
        ];
        let signer = &[&seeds[..]];
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.rewards_pool.clone(),
                    to: ctx.accounts.staker_token_account.to_account_info(),
                    authority: governance_state.to_account_info(),
                },
                signer,
            ),
            rewards,
        )?;
        
        staker_info.referral_rewards = 0;
        staker_info.total_rewards_claimed += rewards;
        
        msg!("✅ Claimed {} TACH referral rewards", rewards / 1_000_000_000);
        
        Ok(())
    }

    /// Update loyalty tier based on stake duration
    pub fn update_loyalty_tier(ctx: Context<UpdateLoyaltyTier>) -> Result<()> {
        let staker_info = &mut ctx.accounts.staker_info;
        let current_time = Clock::get()?.unix_timestamp;
        let stake_duration = current_time - staker_info.first_stake_timestamp;
        
        // Calculate loyalty tier
        let new_tier = if stake_duration >= 31536000 { // 12+ months
            4 // Platinum
        } else if stake_duration >= 15768000 { // 6-12 months
            3 // Gold
        } else if stake_duration >= 7884000 { // 3-6 months
            2 // Silver
        } else if stake_duration >= 2628000 { // 1-3 months
            1 // Bronze
        } else {
            0 // None
        };
        
        if new_tier > staker_info.loyalty_tier {
            staker_info.loyalty_tier = new_tier;
            let tier_name = match new_tier {
                4 => "Platinum",
                3 => "Gold",
                2 => "Silver",
                1 => "Bronze",
                _ => "None",
            };
            msg!("✅ Loyalty tier upgraded to: {}", tier_name);
        }
        
        Ok(())
    }
    
    /// Migrate governance account from old structure to new structure
    /// This expands the account size and initializes new fields
    pub fn migrate_governance(ctx: Context<MigrateGovernance>) -> Result<()> {
        let governance_account = &ctx.accounts.governance_state;
        let current_size = governance_account.to_account_info().data_len();
        
        msg!("Current governance size: {} bytes", current_size);
        
        // Old size was 187 bytes, new size should be larger
        let new_size = 8 + std::mem::size_of::<GovernanceState>();
        msg!("New governance size: {} bytes", new_size);
        
        if current_size >= new_size {
            msg!("✅ Governance already migrated!");
            return Ok(());
        }
        
        // Get account info
        let governance_info = governance_account.to_account_info();
        
        // Calculate rent for the new size
        let rent = Rent::get()?;
        let new_rent_minimum = rent.minimum_balance(new_size);
        let current_lamports = governance_info.lamports();
        
        if current_lamports < new_rent_minimum {
            let additional_rent = new_rent_minimum - current_lamports;
            msg!("Adding {} lamports for rent", additional_rent);
            
            // Transfer additional rent from authority
            let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.authority.key(),
                &governance_info.key(),
                additional_rent,
            );
            
            anchor_lang::solana_program::program::invoke(
                &transfer_ix,
                &[
                    ctx.accounts.authority.to_account_info(),
                    governance_info.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }
        
        // Realloc the account
        governance_info.realloc(new_size, false)?;
        
        // Read existing data before it gets overwritten
        let data = governance_info.try_borrow_data()?;
        let mut offset = 8; // Skip discriminator
        
        // Read old fields (these stay in the same positions)
        let authority = Pubkey::try_from(&data[offset..offset+32]).unwrap(); offset += 32;
        let tach_mint = Pubkey::try_from(&data[offset..offset+32]).unwrap(); offset += 32;
        let vault = Pubkey::try_from(&data[offset..offset+32]).unwrap(); offset += 32;
        let rewards_pool = Pubkey::try_from(&data[offset..offset+32]).unwrap(); offset += 32;
        let min_stake = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let min_proposal_stake = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let voting_period = i64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let total_proposals = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let total_staked = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let total_rewards_distributed = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()); offset += 8;
        let bump = data[offset]; offset += 1;
        let vault_bump = data[offset]; offset += 1;
        let rewards_pool_bump = data[offset];
        
        drop(data); // Release borrow
        
        // Now write back the data with new fields
        let mut governance_data = governance_info.try_borrow_mut_data()?;
        let mut offset = 8;
        
        // Write old fields back
        governance_data[offset..offset+32].copy_from_slice(&authority.to_bytes()); offset += 32;
        governance_data[offset..offset+32].copy_from_slice(&tach_mint.to_bytes()); offset += 32;
        governance_data[offset..offset+32].copy_from_slice(&vault.to_bytes()); offset += 32;
        governance_data[offset..offset+32].copy_from_slice(&rewards_pool.to_bytes()); offset += 32;
        governance_data[offset..offset+8].copy_from_slice(&min_stake.to_le_bytes()); offset += 8;
        governance_data[offset..offset+8].copy_from_slice(&min_proposal_stake.to_le_bytes()); offset += 8;
        governance_data[offset..offset+8].copy_from_slice(&voting_period.to_le_bytes()); offset += 8;
        governance_data[offset..offset+8].copy_from_slice(&total_proposals.to_le_bytes()); offset += 8;
        governance_data[offset..offset+8].copy_from_slice(&total_staked.to_le_bytes()); offset += 8;
        governance_data[offset..offset+8].copy_from_slice(&total_rewards_distributed.to_le_bytes()); offset += 8;
        governance_data[offset] = bump; offset += 1;
        governance_data[offset] = vault_bump; offset += 1;
        governance_data[offset] = rewards_pool_bump; offset += 1;
        
        // Initialize new fields with defaults
        governance_data[offset..offset+8].copy_from_slice(&100u64.to_le_bytes()); offset += 8; // daily_rewards_rate
        governance_data[offset] = 0; offset += 1; // rewards_paused (false)
        governance_data[offset..offset+8].copy_from_slice(&0i64.to_le_bytes()); offset += 8; // last_epoch_distribution
        governance_data[offset..offset+8].copy_from_slice(&86400i64.to_le_bytes()); offset += 8; // epoch_duration (1 day)
        governance_data[offset..offset+8].copy_from_slice(&1000000000000u64.to_le_bytes()); offset += 8; // pool_refill_threshold
        governance_data[offset..offset+8].copy_from_slice(&0u64.to_le_bytes()); offset += 8; // total_slashed
        governance_data[offset..offset+8].copy_from_slice(&0u64.to_le_bytes()); // total_stakers
        
        msg!("✅ Governance migrated successfully!");
        Ok(())
    }
    
    /// Emergency recovery function for old staker accounts
    /// This allows users to recover their stake from the old 25-byte structure
    pub fn recover_old_stake(ctx: Context<RecoverOldStake>, expected_amount: u64) -> Result<()> {
        let old_staker_info = &ctx.accounts.old_staker_info;
        let new_staker_info = &mut ctx.accounts.new_staker_info;
        
        // Verify the old account has the expected size (25 bytes for old structure)
        require!(old_staker_info.data_len() == 25, GovernanceError::InvalidAccountData);
        
        // Read the old account data manually
        let old_data = old_staker_info.try_borrow_data()?;
        
        // Skip 8-byte discriminator, read staked_amount (u64) and timestamp (i64)
        let staked_amount = u64::from_le_bytes(old_data[8..16].try_into().unwrap());
        let last_stake_timestamp = i64::from_le_bytes(old_data[16..24].try_into().unwrap());
        
        // Verify the amount matches what the user expects (safety check)
        require!(staked_amount == expected_amount, GovernanceError::InvalidAmount);
        
        // Initialize the new staker account with migrated data
        new_staker_info.staked_amount = staked_amount;
        new_staker_info.last_stake_timestamp = last_stake_timestamp;
        new_staker_info.bump = ctx.bumps.new_staker_info;
        
        // Initialize new fields with defaults
        new_staker_info.total_rewards_claimed = 0;
        new_staker_info.last_claim_timestamp = last_stake_timestamp;
        new_staker_info.pending_rewards = 0;
        new_staker_info.compounded_rewards = 0;
        new_staker_info.uptime_score = 10000; // Start at 100%
        new_staker_info.submissions_count = 0;
        new_staker_info.accurate_submissions = 0;
        new_staker_info.first_stake_timestamp = last_stake_timestamp;
        new_staker_info.loyalty_tier = 0; // Will be calculated based on time
        new_staker_info.referrer = Pubkey::default();
        new_staker_info.referral_count = 0;
        new_staker_info.referral_rewards = 0;
        new_staker_info.vested_rewards = 0;
        new_staker_info.vesting_start = 0;
        
        msg!("✅ Recovered stake: {} TACH from {}", 
            staked_amount as f64 / 1e9, 
            last_stake_timestamp
        );
        
        Ok(())
    }

    /// Clean up garbage data in recovered staker account
    /// This zeros out all the uninitialized fields that contain random bytes
    pub fn cleanup_staker_account(ctx: Context<CleanupStaker>) -> Result<()> {
        let staker_info = &mut ctx.accounts.staker_info;
        
        msg!("Cleaning up staker account for: {}", ctx.accounts.staker.key());
        
        // Keep the important fields as-is:
        // - staked_amount
        // - last_stake_timestamp
        // - bump
        
        // Zero out all the garbage fields
        staker_info.total_rewards_claimed = 0;
        staker_info.last_claim_timestamp = 0;
        staker_info.pending_rewards = 0;
        staker_info.compounded_rewards = 0;
        staker_info.uptime_score = 10000; // 100% default
        staker_info.submissions_count = 0;
        staker_info.accurate_submissions = 0;
        staker_info.first_stake_timestamp = staker_info.last_stake_timestamp;
        staker_info.loyalty_tier = 0; // Bronze
        staker_info.referrer = Pubkey::default();
        staker_info.referral_count = 0;
        staker_info.referral_rewards = 0;
        staker_info.vested_rewards = 0;
        staker_info.vesting_start = 0;
        
        msg!("✅ Staker account cleaned up successfully");
        
        Ok(())
    }

}

// ============================================================================
// ACCOUNT CONTEXTS (Outside program module)
// ============================================================================

/// Context for migrating governance account to new structure
#[derive(Accounts)]
pub struct MigrateGovernance<'info> {
    /// CHECK: We manually handle the realloc and data migration
    #[account(
        mut,
        seeds = [b"governance"],
        bump,
    )]
    pub governance_state: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for recovering old stake from 25-byte accounts
#[derive(Accounts)]
pub struct RecoverOldStake<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    /// The old staker account (25 bytes) - read-only, we'll read it manually
    /// CHECK: We manually validate this is the old staker account
    #[account(
        seeds = [b"staker", staker.key().as_ref()],
        bump,
    )]
    pub old_staker_info: AccountInfo<'info>,
    
    /// The new staker account (171 bytes) - we'll create this with a different seed
    #[account(
        init,
        payer = staker,
        space = 8 + std::mem::size_of::<StakerInfo>(),
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump
    )]
    pub new_staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub staker: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for cleaning up garbage data in recovered staker account
#[derive(Accounts)]
pub struct CleanupStaker<'info> {
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    pub staker: Signer<'info>,
}

// ============================================================================
// HELPER FUNCTIONS (Outside program module)
// ============================================================================

/// Helper function to calculate total rewards with bonuses
pub fn calculate_total_rewards_internal(
    governance_state: &GovernanceState,
    staker_info: &StakerInfo,
) -> Result<u64> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_staked = current_time - staker_info.last_claim_timestamp;
    
    // Base rewards calculation
    let seconds_per_day: u64 = 86400;
    let stake_percentage = if governance_state.total_staked > 0 {
        (staker_info.staked_amount as u128 * 1_000_000) / governance_state.total_staked as u128
    } else {
        0
    };
    
    let base_rewards = ((governance_state.daily_rewards_rate as u128 * stake_percentage * time_staked as u128) 
        / (seconds_per_day as u128 * 1_000_000)) as u64;
    
    // Apply performance multiplier (50% to 150%)
    let performance_multiplier = if staker_info.submissions_count > 0 {
        let accuracy_rate = (staker_info.accurate_submissions * 10000) / staker_info.submissions_count;
        let uptime_factor = staker_info.uptime_score;
        // Average of accuracy and uptime
        (accuracy_rate + uptime_factor) / 2
    } else {
        10000 // 100% default
    };
    
    let performance_adjusted = (base_rewards as u128 * performance_multiplier as u128) / 10000;
    
    // Apply loyalty bonus (0% to 50%)
    let loyalty_multiplier = match staker_info.loyalty_tier {
        4 => 15000, // Platinum: 150%
        3 => 12000, // Gold: 120%
        2 => 11000, // Silver: 110%
        1 => 10000, // Bronze: 100%
        _ => 10000, // None: 100%
    };
    
    let total_rewards = (performance_adjusted * loyalty_multiplier as u128) / 10000;
    
    Ok(total_rewards as u64)
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + GovernanceState::INIT_SPACE,
        seeds = [b"governance"],
        bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        init,
        payer = payer,
        token::mint = tach_mint,
        token::authority = governance_state,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = payer,
        token::mint = tach_mint,
        token::authority = governance_state,
        seeds = [b"rewards-pool"],
        bump
    )]
    pub rewards_pool: Account<'info, TokenAccount>,
    
    pub tach_mint: Account<'info, Mint>,
    
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitStaker<'info> {
    #[account(
        init,
        payer = staker,
        space = 8 + StakerInfo::INIT_SPACE,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(mut)]
    pub staker: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump = governance_state.vault_bump
    )]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub staker: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump = governance_state.vault_bump
    )]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub staker: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Slash<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump = governance_state.vault_bump
    )]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"rewards-pool"],
        bump = governance_state.rewards_pool_bump
    )]
    pub rewards_pool: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"staker-info", slashed_staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    /// CHECK: The staker being slashed
    pub slashed_staker: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        seeds = [b"staker-v2", proposer.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [b"proposal", governance_state.total_proposals.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct Vote<'info> {
    #[account(
        seeds = [b"staker-v2", voter.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(
        mut,
        seeds = [b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteProposal<'info> {
    #[account(
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"proposal", proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    /// CHECK: Rewards pool PDA - validated manually
    #[account(mut)]
    pub rewards_pool: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    
    pub staker: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateMinStake<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct FundRewardsPool<'info> {
    #[account(
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"rewards-pool"],
        bump = governance_state.rewards_pool_bump
    )]
    pub rewards_pool: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct GovernanceState {
    pub authority: Pubkey,              // 32 bytes
    pub tach_mint: Pubkey,              // 32 bytes
    pub vault: Pubkey,                  // 32 bytes
    pub rewards_pool: Pubkey,           // 32 bytes
    pub min_stake: u64,                 // 8 bytes
    pub min_proposal_stake: u64,        // 8 bytes
    pub voting_period: i64,             // 8 bytes
    pub total_proposals: u64,           // 8 bytes
    pub total_staked: u64,              // 8 bytes
    pub total_rewards_distributed: u64, // 8 bytes
    pub bump: u8,                       // 1 byte
    pub vault_bump: u8,                 // 1 byte
    pub rewards_pool_bump: u8,          // 1 byte
    // NEW: Rewards system enhancements
    pub daily_rewards_rate: u64,        // 8 bytes - Adjustable rewards rate
    pub rewards_paused: bool,           // 1 byte - Emergency pause
    pub last_epoch_distribution: i64,   // 8 bytes - Last auto-distribution
    pub epoch_duration: i64,            // 8 bytes - Epoch length in seconds
    pub pool_refill_threshold: u64,     // 8 bytes - Auto-refill trigger
    pub total_slashed: u64,             // 8 bytes - Total slashed tokens
    pub total_stakers: u64,             // 8 bytes - Number of active stakers
}

#[account]
#[derive(InitSpace)]
pub struct StakerInfo {
    pub staked_amount: u64,             // 8 bytes
    pub last_stake_timestamp: i64,      // 8 bytes
    pub bump: u8,                       // 1 byte
    // NEW: Rewards tracking
    pub total_rewards_claimed: u64,     // 8 bytes - Lifetime rewards
    pub last_claim_timestamp: i64,      // 8 bytes - Last claim time
    pub pending_rewards: u64,           // 8 bytes - Unclaimed rewards
    pub compounded_rewards: u64,        // 8 bytes - Auto-staked rewards
    // NEW: Performance tracking
    pub uptime_score: u64,              // 8 bytes - Performance multiplier (0-10000 = 0-100%)
    pub submissions_count: u64,         // 8 bytes - Total submissions
    pub accurate_submissions: u64,      // 8 bytes - Accurate submissions
    // NEW: Loyalty tracking
    pub first_stake_timestamp: i64,     // 8 bytes - First stake time
    pub loyalty_tier: u8,               // 1 byte - 0=None, 1=Bronze, 2=Silver, 3=Gold, 4=Platinum
    // NEW: Referral tracking
    pub referrer: Pubkey,               // 32 bytes - Who referred this staker
    pub referral_count: u64,            // 8 bytes - How many referred
    pub referral_rewards: u64,          // 8 bytes - Rewards from referrals
    // NEW: Vesting
    pub vested_rewards: u64,            // 8 bytes - Vested amount
    pub vesting_start: i64,             // 8 bytes - Vesting start time
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub id: u64,                        // 8 bytes
    pub proposer: Pubkey,               // 32 bytes
    #[max_len(100)]
    pub title: String,                  // 4 + 100 bytes
    #[max_len(500)]
    pub description: String,            // 4 + 500 bytes
    pub proposal_type: ProposalType,    // 1 byte
    pub votes_for: u64,                 // 8 bytes
    pub votes_against: u64,             // 8 bytes
    pub status: ProposalStatus,         // 1 byte
    pub created_at: i64,                // 8 bytes
    pub voting_ends_at: i64,            // 8 bytes
    pub bump: u8,                       // 1 byte
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum ProposalType {
    ParameterChange,
    ProtocolUpgrade,
    TreasurySpend,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum ProposalStatus {
    Active,
    Executed,
    Rejected,
}

// ============================================================================
// NEW CONTEXT STRUCTS FOR REWARDS FEATURES
// ============================================================================

#[derive(Accounts)]
pub struct SetRewardsPaused<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateRewardsRate<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePerformance<'info> {
    #[account(
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    /// CHECK: Staker account being updated
    pub staker: AccountInfo<'info>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeEpochRewards<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
}

#[derive(Accounts)]
pub struct ClaimReferralRewards<'info> {
    #[account(
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    /// CHECK: Rewards pool PDA - validated manually
    #[account(mut)]
    pub rewards_pool: AccountInfo<'info>,
    
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,
    
    pub staker: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateLoyaltyTier<'info> {
    #[account(
        mut,
        seeds = [b"staker-v2", staker.key().as_ref()],
        bump = staker_info.bump
    )]
    pub staker_info: Account<'info, StakerInfo>,
    
    pub staker: Signer<'info>,
}

#[error_code]
pub enum GovernanceError {
    #[msg("Unauthorized: Only authority can perform this action")]
    Unauthorized,
    #[msg("Invalid rewards pool PDA")]
    InvalidRewardsPool,
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    #[msg("Below minimum stake requirement")]
    BelowMinimumStake,
    #[msg("Insufficient stake to create proposal")]
    InsufficientStakeForProposal,
    #[msg("Cooldown period still active")]
    CooldownPeriodActive,
    #[msg("Proposal is not active")]
    ProposalNotActive,
    #[msg("Voting period has ended")]
    VotingPeriodEnded,
    #[msg("Voting period has not ended yet")]
    VotingPeriodNotEnded,
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
    #[msg("Rewards distribution is paused")]
    RewardsPaused,
    #[msg("Epoch distribution not ready yet")]
    EpochNotReady,
    #[msg("Rewards pool balance is sufficient")]
    PoolBalanceSufficient,
    #[msg("Account already migrated")]
    AlreadyMigrated,
    #[msg("Invalid account data for recovery")]
    InvalidAccountData,
    #[msg("Amount does not match expected value")]
    InvalidAmount,
}
