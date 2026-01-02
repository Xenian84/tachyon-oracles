use anchor_lang::prelude::*;

declare_id!("PFEDu3nNzRQQYmX1Xvso2BxtPbUQaZEVoiLbXDy6U3W");

#[program]
pub mod tachyon_price_feeds {
    use super::*;

    /// Initialize a new price feed
    pub fn initialize_feed(
        ctx: Context<InitializeFeed>,
        symbol: String,
        description: String,
        decimals: u8,
    ) -> Result<()> {
        require!(symbol.len() <= 32, PriceFeedError::SymbolTooLong);
        require!(description.len() <= 128, PriceFeedError::DescriptionTooLong);
        
        let feed = &mut ctx.accounts.price_feed;
        feed.authority = ctx.accounts.authority.key();
        feed.symbol = symbol.clone();
        feed.description = description;
        feed.decimals = decimals;
        feed.price = 0;
        feed.confidence = 0;
        feed.expo = 0;
        feed.last_update = 0;
        feed.publisher_count = 0;
        feed.status = FeedStatus::Inactive as u8;
        feed.bump = ctx.bumps.price_feed;
        
        msg!("Price feed initialized: {}", symbol);
        Ok(())
    }

    /// Update price feed (called by oracle nodes)
    pub fn update_price(
        ctx: Context<UpdatePrice>,
        price: i64,
        confidence: u64,
        expo: i32,
        publisher: Pubkey,
    ) -> Result<()> {
        let feed = &mut ctx.accounts.price_feed;
        let clock = Clock::get()?;
        
        // Verify submitter is authorized (from governance)
        require!(
            ctx.accounts.submitter.key() == feed.authority || 
            ctx.accounts.governance_state.is_some(),
            PriceFeedError::Unauthorized
        );
        
        // Update price data
        feed.price = price;
        feed.confidence = confidence;
        feed.expo = expo;
        feed.last_update = clock.unix_timestamp;
        feed.status = FeedStatus::Active as u8;
        
        // Emit event for indexers
        emit!(PriceUpdated {
            symbol: feed.symbol.clone(),
            price,
            confidence,
            expo,
            publisher,
            timestamp: clock.unix_timestamp,
        });
        
        msg!(
            "Price updated: {} = {} (conf: {}, expo: {})",
            feed.symbol,
            price,
            confidence,
            expo
        );
        
        Ok(())
    }

    /// Aggregate prices from multiple publishers
    pub fn aggregate_prices(
        ctx: Context<AggregatePrices>,
        prices: Vec<PriceSubmission>,
    ) -> Result<()> {
        let feed = &mut ctx.accounts.price_feed;
        let clock = Clock::get()?;
        
        require!(!prices.is_empty(), PriceFeedError::NoPrices);
        require!(prices.len() <= 100, PriceFeedError::TooManyPrices);
        
        // Calculate median price
        let mut sorted_prices: Vec<i64> = prices.iter().map(|p| p.price).collect();
        sorted_prices.sort();
        let median_price = sorted_prices[sorted_prices.len() / 2];
        
        // Calculate confidence (standard deviation)
        let mean: i64 = sorted_prices.iter().sum::<i64>() / sorted_prices.len() as i64;
        let variance: u64 = sorted_prices
            .iter()
            .map(|p| {
                let diff = (*p - mean).abs() as u64;
                diff * diff
            })
            .sum::<u64>() / sorted_prices.len() as u64;
        let confidence = (variance as f64).sqrt() as u64;
        
        // Update feed
        feed.price = median_price;
        feed.confidence = confidence;
        feed.expo = prices[0].expo; // Assume all have same exponent
        feed.last_update = clock.unix_timestamp;
        feed.publisher_count = prices.len() as u32;
        feed.status = FeedStatus::Active as u8;
        
        // Emit aggregated price event
        emit!(PriceAggregated {
            symbol: feed.symbol.clone(),
            price: median_price,
            confidence,
            publisher_count: prices.len() as u32,
            timestamp: clock.unix_timestamp,
        });
        
        msg!(
            "Aggregated {} prices for {}: {} ± {}",
            prices.len(),
            feed.symbol,
            median_price,
            confidence
        );
        
        Ok(())
    }

    /// Update feed status
    pub fn update_status(
        ctx: Context<UpdateStatus>,
        status: u8,
    ) -> Result<()> {
        let feed = &mut ctx.accounts.price_feed;
        
        require!(status <= 2, PriceFeedError::InvalidStatus);
        
        feed.status = status;
        
        msg!("Feed {} status updated to {}", feed.symbol, status);
        Ok(())
    }

    /// Get current price
    pub fn get_price(ctx: Context<GetPrice>) -> Result<PriceData> {
        let feed = &ctx.accounts.price_feed;
        
        Ok(PriceData {
            symbol: feed.symbol.clone(),
            price: feed.price,
            confidence: feed.confidence,
            expo: feed.expo,
            last_update: feed.last_update,
            publisher_count: feed.publisher_count,
            status: feed.status,
        })
    }
}

// Accounts

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct InitializeFeed<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + PriceFeed::INIT_SPACE,
        seeds = [b"price-feed", symbol.as_bytes()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
    
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(
        mut,
        seeds = [b"price-feed", price_feed.symbol.as_bytes()],
        bump = price_feed.bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
    
    pub submitter: Signer<'info>,
    
    /// Optional: Governance state to verify submitter is staked validator
    pub governance_state: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct AggregatePrices<'info> {
    #[account(
        mut,
        seeds = [b"price-feed", price_feed.symbol.as_bytes()],
        bump = price_feed.bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateStatus<'info> {
    #[account(
        mut,
        seeds = [b"price-feed", price_feed.symbol.as_bytes()],
        bump = price_feed.bump,
        has_one = authority
    )]
    pub price_feed: Account<'info, PriceFeed>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    #[account(
        seeds = [b"price-feed", price_feed.symbol.as_bytes()],
        bump = price_feed.bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
}

// State

#[account]
#[derive(InitSpace)]
pub struct PriceFeed {
    pub authority: Pubkey,              // 32 bytes
    #[max_len(32)]
    pub symbol: String,                 // 4 + 32 bytes (e.g., "BTC/USD")
    #[max_len(128)]
    pub description: String,            // 4 + 128 bytes
    pub decimals: u8,                   // 1 byte
    pub price: i64,                     // 8 bytes - Current price
    pub confidence: u64,                // 8 bytes - Confidence interval
    pub expo: i32,                      // 4 bytes - Price exponent
    pub last_update: i64,               // 8 bytes - Last update timestamp
    pub publisher_count: u32,           // 4 bytes - Number of publishers
    pub status: u8,                     // 1 byte - 0=Inactive, 1=Active, 2=Deprecated
    pub bump: u8,                       // 1 byte
}

// Data structures

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceSubmission {
    pub publisher: Pubkey,
    pub price: i64,
    pub confidence: u64,
    pub expo: i32,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    pub symbol: String,
    pub price: i64,
    pub confidence: u64,
    pub expo: i32,
    pub last_update: i64,
    pub publisher_count: u32,
    pub status: u8,
}

// Events

#[event]
pub struct PriceUpdated {
    pub symbol: String,
    pub price: i64,
    pub confidence: u64,
    pub expo: i32,
    pub publisher: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PriceAggregated {
    pub symbol: String,
    pub price: i64,
    pub confidence: u64,
    pub publisher_count: u32,
    pub timestamp: i64,
}

// Enums

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum FeedStatus {
    Inactive = 0,
    Active = 1,
    Deprecated = 2,
}

// Errors

#[error_code]
pub enum PriceFeedError {
    #[msg("Symbol too long (max 32 characters)")]
    SymbolTooLong,
    
    #[msg("Description too long (max 128 characters)")]
    DescriptionTooLong,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("No prices provided")]
    NoPrices,
    
    #[msg("Too many prices (max 100)")]
    TooManyPrices,
    
    #[msg("Invalid status")]
    InvalidStatus,
    
    #[msg("Feed is inactive")]
    FeedInactive,
}

