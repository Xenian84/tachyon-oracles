#![allow(dead_code)]
use std::sync::Arc;
use solana_sdk::signer::Signer;
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc;
use tracing::{info, debug, warn};

use crate::aggregator::MerkleBatch;
use crate::config::NodeConfig;

// Tower BFT for production-grade consensus
pub mod oracle_tower;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub batch: MerkleBatch,
    pub votes: HashMap<String, Vote>,
    pub consensus_root: Option<String>,
    pub agreeing_stake: u64,
    pub total_stake: u64,
    pub is_leader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub node_pubkey: String,
    pub root_hash: String,
    pub stake: u64,
    pub signature: Vec<u8>,
}

pub async fn start_consensus(
    config: Arc<NodeConfig>,
    mut batch_rx: mpsc::Receiver<MerkleBatch>,
    consensus_tx: mpsc::Sender<ConsensusResult>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    info!("🗳️  Starting consensus module with stake-weighted voting...");
    
    let node_pubkey = config.identity.pubkey().to_string();
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    loop {
        tokio::select! {
            Some(batch) = batch_rx.recv() => {
                debug!("🗳️  Processing batch with root: {}", &batch.root[..8]);
                
                // 1. Get current slot
                let current_slot = match rpc_client.get_slot() {
                    Ok(slot) => slot,
                    Err(e) => {
                        warn!("Failed to get current slot: {}", e);
                        0
                    }
                };
                
                // 2. Query all stakers from governance
                let (validators, total_stake) = match query_validators(&config, &rpc_client).await {
                    Ok(result) => result,
                    Err(e) => {
                        warn!("Failed to query validators: {}", e);
                        // Fallback: assume we're the only validator
                        (vec![(node_pubkey.clone(), 100_000_000_000_000u64)], 100_000_000_000_000u64)
                    }
                };
                
                debug!("🗳️  Found {} validators with total stake: {}", validators.len(), total_stake);
                
                // 3. Broadcast our root to peers (via gossip)
                // TODO: Implement actual gossip broadcasting
                // For now, we simulate by assuming all nodes agree
                
                // 4. Collect votes (in production, from gossip)
                let mut votes = HashMap::new();
                
                // Add our own vote
                let our_stake = validators.iter()
                    .find(|(pubkey, _)| pubkey == &node_pubkey)
                    .map(|(_, stake)| *stake)
                    .unwrap_or(0);
                
                votes.insert(node_pubkey.clone(), Vote {
                    node_pubkey: node_pubkey.clone(),
                    root_hash: batch.root.clone(),
                    stake: our_stake,
                    signature: vec![], // TODO: Sign the root
                });
                
                // 5. Tally votes and check for 2/3 consensus
                let (consensus_root, agreeing_stake) = tally_votes(&votes, total_stake);
                
                // 6. Determine if we're the leader for this slot
                let is_leader = match select_leader(&validators, current_slot) {
                    Some(leader_pubkey) => leader_pubkey == node_pubkey,
                    None => false,
                };
                
                if consensus_root.is_some() {
                    info!("✅ Consensus reached: {}/{} stake agrees", agreeing_stake, total_stake);
                } else {
                    warn!("❌ No consensus: need 2/3 stake agreement");
                }
                
                if is_leader {
                    info!("👑 We are the leader for slot {}", current_slot);
                } else {
                    debug!("   Not the leader for this slot");
                }
                
                let result = ConsensusResult {
                    batch,
                    votes,
                    consensus_root,
                    agreeing_stake,
                    total_stake,
                    is_leader,
                };
                
                if let Err(e) = consensus_tx.send(result).await {
                    tracing::error!("Failed to send consensus result: {}", e);
                }
            }
            _ = shutdown.recv() => {
                info!("🗳️  Consensus module shutting down...");
                break;
            }
        }
    }
    
    Ok(())
}

// Query all validators and their stakes from TachyonGovernance
async fn query_validators(config: &NodeConfig, rpc_client: &RpcClient) -> Result<(Vec<(String, u64)>, u64)> {
    let governance_program = Pubkey::from_str(&config.program_id)?;
    
    // In production, we would query all staker accounts
    // For now, simplified: just check if we're staked
    // Use "staker-v2" seed for the new account structure
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let mut validators = Vec::new();
    let mut total_stake = 0u64;
    
    // Check our stake
    match rpc_client.get_account(&staker_info_pda) {
        Ok(account) => {
            // Parse stake amount from the account data
            // StakerInfo structure: discriminator (8) + staked_amount (8) + ...
            if account.data.len() >= 16 {
                let stake_bytes: [u8; 8] = account.data[8..16].try_into().unwrap();
                let stake = u64::from_le_bytes(stake_bytes);
                info!("✅ Found our stake: {} TACH", stake as f64 / 1e9);
                validators.push((config.identity.pubkey().to_string(), stake));
                total_stake += stake;
            } else {
                warn!("Staker account too small, cannot read stake");
            }
        }
        Err(_) => {
            warn!("Node not staked, cannot participate in consensus");
        }
    }
    
    // TODO: Query other validators from on-chain data
    // This would involve:
    // 1. Getting all staker-info accounts
    // 2. Parsing their stake amounts
    // 3. Building the validator list
    
    Ok((validators, total_stake))
}

// Tally votes and return consensus root if 2/3 agreement reached
fn tally_votes(votes: &HashMap<String, Vote>, total_stake: u64) -> (Option<String>, u64) {
    let mut root_stakes: HashMap<String, u64> = HashMap::new();
    
    // Group votes by root hash
    for vote in votes.values() {
        *root_stakes.entry(vote.root_hash.clone()).or_insert(0) += vote.stake;
    }
    
    // Find root with most stake
    let quorum_threshold = (total_stake * 2) / 3;
    
    for (root, stake) in root_stakes.iter() {
        if *stake >= quorum_threshold {
            return (Some(root.clone()), *stake);
        }
    }
    
    // No consensus
    (None, 0)
}

// Stake-weighted leader selection (deterministic based on slot)
fn select_leader(validators: &[(String, u64)], slot: u64) -> Option<String> {
    if validators.is_empty() {
        return None;
    }
    
    let total_stake: u64 = validators.iter().map(|(_, stake)| stake).sum();
    if total_stake == 0 {
        return None;
    }
    
    // Use slot as seed for deterministic selection
    // This ensures all nodes select the same leader for a given slot
    let target = (slot * 12345) % total_stake;
    
    let mut cumulative = 0u64;
    for (pubkey, stake) in validators {
        cumulative += stake;
        if cumulative > target {
            return Some(pubkey.clone());
        }
    }
    
    validators.first().map(|(pubkey, _)| pubkey.clone())
}

// Verify that 2/3 of stake voted for the same root
pub fn verify_quorum(votes: &HashMap<String, Vote>, total_stake: u64) -> bool {
    let mut root_stakes: HashMap<String, u64> = HashMap::new();
    
    for vote in votes.values() {
        *root_stakes.entry(vote.root_hash.clone()).or_insert(0) += vote.stake;
    }
    
    // Check if any root has 2/3+ stake
    let quorum_threshold = (total_stake * 2) / 3;
    root_stakes.values().any(|&stake| stake >= quorum_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_leader_selection() {
        let validators = vec![
            ("validator1".to_string(), 100),
            ("validator2".to_string(), 200),
            ("validator3".to_string(), 300),
        ];
        
        // Same slot should always select same leader
        let leader1 = select_leader(&validators, 100);
        let leader2 = select_leader(&validators, 100);
        assert_eq!(leader1, leader2);
        
        // Different slots may select different leaders
        let leader_slot_1 = select_leader(&validators, 1);
        let leader_slot_2 = select_leader(&validators, 2);
        assert!(leader_slot_1.is_some());
        assert!(leader_slot_2.is_some());
    }
    
    #[test]
    fn test_quorum_verification() {
        let mut votes = HashMap::new();
        
        votes.insert("v1".to_string(), Vote {
            node_pubkey: "v1".to_string(),
            root_hash: "root1".to_string(),
            stake: 200,
            signature: vec![],
        });
        
        votes.insert("v2".to_string(), Vote {
            node_pubkey: "v2".to_string(),
            root_hash: "root1".to_string(),
            stake: 100,
        signature: vec![],
        });
        
        // 300/400 = 75% > 66.67%, should reach quorum
        assert!(verify_quorum(&votes, 400));
        
        // 300/500 = 60% < 66.67%, should not reach quorum
        assert!(!verify_quorum(&votes, 500));
    }
}
