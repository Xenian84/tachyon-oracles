use anchor_lang::prelude::*;
use solana_program::keccak;

declare_id!("VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR");

/// TachyonVerifier - Proof verification for price feeds
/// 
/// This contract provides optimized Merkle proof verification
/// for DeFi protocols consuming Tachyon oracle data.
#[program]
pub mod tachyon_verifier {
    use super::*;

    /// Verify a Merkle proof and return verified price data
    pub fn verify_price(
        ctx: Context<VerifyPrice>,
        asset_id: [u8; 32],
        price: i64,
        confidence: i64,
        timestamp: i64,
        merkle_root: [u8; 32],
        proof: Vec<[u8; 32]>,
    ) -> Result<VerifiedPrice> {
        // Serialize the price feed (same format as L2 aggregator)
        let mut leaf_data = Vec::with_capacity(56);
        leaf_data.extend_from_slice(&asset_id);
        leaf_data.extend_from_slice(&price.to_le_bytes());
        leaf_data.extend_from_slice(&confidence.to_le_bytes());
        leaf_data.extend_from_slice(&timestamp.to_le_bytes());
        
        // Hash the leaf
        let mut current_hash = keccak::hash(&leaf_data).to_bytes();
        
        // Verify proof by climbing the tree
        for proof_element in proof.iter() {
            current_hash = if current_hash < *proof_element {
                keccak::hash(&[&current_hash[..], &proof_element[..]].concat()).to_bytes()
            } else {
                keccak::hash(&[&proof_element[..], &current_hash[..]].concat()).to_bytes()
            };
        }
        
        // Check if computed root matches provided root
        require!(
            current_hash == merkle_root,
            VerifierError::InvalidProof
        );
        
        // Check timestamp freshness (not older than 60 seconds)
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time - timestamp < 60,
            VerifierError::StalePrice
        );
        
        msg!(
            "✅ Price verified: asset={:?}, price={}, conf={}",
            &asset_id[..8],
            price,
            confidence
        );
        
        Ok(VerifiedPrice {
            asset_id,
            price,
            confidence,
            timestamp,
            verified_at: current_time,
            is_valid: true,
        })
    }

    /// Batch verify multiple prices (gas optimization)
    pub fn verify_batch(
        ctx: Context<VerifyBatch>,
        prices: Vec<PriceData>,
        merkle_root: [u8; 32],
        proofs: Vec<Vec<[u8; 32]>>,
    ) -> Result<Vec<bool>> {
        require!(
            prices.len() == proofs.len(),
            VerifierError::MismatchedInputs
        );
        
        let mut results = Vec::with_capacity(prices.len());
        
        for (price_data, proof) in prices.iter().zip(proofs.iter()) {
            // Serialize the price feed
            let mut leaf_data = Vec::with_capacity(56);
            leaf_data.extend_from_slice(&price_data.asset_id);
            leaf_data.extend_from_slice(&price_data.price.to_le_bytes());
            leaf_data.extend_from_slice(&price_data.confidence.to_le_bytes());
            leaf_data.extend_from_slice(&price_data.timestamp.to_le_bytes());
            
            // Hash the leaf
            let mut current_hash = keccak::hash(&leaf_data).to_bytes();
            
            // Verify proof
            for proof_element in proof.iter() {
                current_hash = if current_hash < *proof_element {
                    keccak::hash(&[&current_hash[..], &proof_element[..]].concat()).to_bytes()
                } else {
                    keccak::hash(&[&proof_element[..], &current_hash[..]].concat()).to_bytes()
                };
            }
            
            // Check if valid
            results.push(current_hash == merkle_root);
        }
        
        msg!("Batch verified: {}/{} valid", results.iter().filter(|&&v| v).count(), results.len());
        
        Ok(results)
    }
}

#[derive(Accounts)]
pub struct VerifyPrice<'info> {
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyBatch<'info> {
    pub payer: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    pub asset_id: [u8; 32],
    pub price: i64,
    pub confidence: i64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VerifiedPrice {
    pub asset_id: [u8; 32],
    pub price: i64,
    pub confidence: i64,
    pub timestamp: i64,
    pub verified_at: i64,
    pub is_valid: bool,
}

#[error_code]
pub enum VerifierError {
    #[msg("Invalid Merkle proof")]
    InvalidProof,
    #[msg("Price data is stale (>60s old)")]
    StalePrice,
    #[msg("Mismatched inputs: prices and proofs length differ")]
    MismatchedInputs,
}

