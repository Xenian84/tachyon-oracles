use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{info, error, warn};
use borsh::BorshSerialize;

use crate::aggregator::FeedData;

const PRICE_FEEDS_PROGRAM_ID: &str = "PFEDu3nNzRQQYmX1Xvso2BxtPbUQaZEVoiLbXDy6U3W";

// Instruction discriminator for update_price (calculated from "global:update_price")
const UPDATE_PRICE_DISCRIMINATOR: [u8; 8] = [
    61, 34, 117, 155, 75, 34, 123, 208
];

/// Submit price updates to the price feeds contract
/// This function is FULLY DYNAMIC - it will submit any symbol that exists in the contract
pub async fn submit_price_feeds(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    feeds: &[FeedData],
) -> anyhow::Result<Vec<String>> {
    let program_id = Pubkey::from_str(PRICE_FEEDS_PROGRAM_ID)?;
    let mut signatures = Vec::new();
    
    for feed in feeds {
        // Try to submit - if the feed doesn't exist on-chain, it will just skip
        match submit_single_feed(rpc_client, keypair, &program_id, feed).await {
            Ok(sig) => {
                if !sig.is_empty() {
                    info!("✅ Submitted {} price: {}", feed.asset_id, sig);
                    signatures.push(sig);
                }
            }
            Err(e) => {
                error!("❌ Failed to submit {} price: {}", feed.asset_id, e);
            }
        }
    }
    
    Ok(signatures)
}

async fn submit_single_feed(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    program_id: &Pubkey,
    feed: &FeedData,
) -> anyhow::Result<String> {
    // Use the symbol directly from the feed data
    // No hardcoded mapping needed!
    let symbol = &feed.asset_id;
    
    // Find price feed PDA
    let (price_feed_pda, _) = Pubkey::find_program_address(
        &[b"price-feed", symbol.as_bytes()],
        program_id,
    );
    
    // Check if the feed exists on-chain
    match rpc_client.get_account(&price_feed_pda) {
        Ok(_) => {
            // Feed exists, proceed with submission
        }
        Err(_) => {
            // Feed doesn't exist on-chain yet
            warn!("⚠️  Price feed {} not initialized on-chain, skipping", symbol);
            return Ok(String::new());
        }
    }
    
    // Build instruction data
    let mut data = Vec::new();
    data.extend_from_slice(&UPDATE_PRICE_DISCRIMINATOR);
    
    // Serialize parameters: price (i64), confidence (u64), expo (i32), publisher (Pubkey)
    feed.price.serialize(&mut data)?;
    (feed.confidence.abs() as u64).serialize(&mut data)?;
    
    // Determine decimals based on symbol
    let expo = get_expo_for_symbol(symbol);
    expo.serialize(&mut data)?;
    
    keypair.pubkey().serialize(&mut data)?;
    
    // Find governance state PDA for authorization
    let governance_program_id = Pubkey::from_str("TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9")?;
    let (governance_state_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program_id,
    );
    
    // Build instruction
    let instruction = Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(price_feed_pda, false),
            AccountMeta::new_readonly(keypair.pubkey(), true),
            AccountMeta::new_readonly(governance_state_pda, false),
        ],
        data,
    };
    
    // Send transaction
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    
    Ok(signature.to_string())
}

/// Get the exponent (decimal places) for a symbol
/// This is the only place that needs updating when adding new symbols
fn get_expo_for_symbol(symbol: &str) -> i32 {
    match symbol {
        // Crypto (8 decimals)
        s if s.starts_with("BTC/") => -8,
        s if s.starts_with("ETH/") => -8,
        s if s.starts_with("SOL/") => -8,
        
        // Native tokens (9 decimals)
        s if s.starts_with("XNT/") => -9,
        s if s.starts_with("TACH/") => -9,
        
        // Forex (6 decimals)
        s if s.starts_with("EUR/") => -6,
        s if s.starts_with("GBP/") => -6,
        s if s.starts_with("JPY/") => -6,
        
        // Commodities (6 decimals)
        s if s.starts_with("XAU/") => -6, // Gold
        s if s.starts_with("XAG/") => -6, // Silver
        s if s.starts_with("WTI/") => -6, // Oil
        
        // Default: 8 decimals
        _ => -8,
    }
}
