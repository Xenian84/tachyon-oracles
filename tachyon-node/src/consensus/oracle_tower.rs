#![allow(dead_code)]
// Oracle Tower - Simplified Tower BFT for Tachyon Oracle Network
// Adapted from Solana Tower BFT for Merkle root consensus

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Batch number (equivalent to Solana's Slot)
pub type BatchNumber = u64;

/// Merkle root hash
pub type MerkleRoot = [u8; 32];

/// Vote for a Merkle root at a specific batch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleVote {
    pub batch_number: BatchNumber,
    pub root: MerkleRoot,
    pub timestamp: i64,
}

/// Vote state tracking for Tower BFT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TowerVoteState {
    /// Recent votes (most recent first)
    pub votes: VecDeque<MerkleVote>,
    /// Root batch (confirmed)
    pub root_batch: Option<BatchNumber>,
    /// Last voted batch
    pub last_voted_batch: Option<BatchNumber>,
}

impl TowerVoteState {
    pub fn new() -> Self {
        Self {
            votes: VecDeque::new(),
            root_batch: None,
            last_voted_batch: None,
        }
    }

    /// Add a new vote
    pub fn push_vote(&mut self, vote: MerkleVote) {
        self.last_voted_batch = Some(vote.batch_number);
        self.votes.push_front(vote);
        
        // Keep only recent votes (last 32)
        if self.votes.len() > 32 {
            self.votes.truncate(32);
        }
    }

    /// Get the last vote
    pub fn last_vote(&self) -> Option<&MerkleVote> {
        self.votes.front()
    }

    /// Check if we've voted for a specific batch
    pub fn has_voted(&self, batch_number: BatchNumber) -> bool {
        self.votes.iter().any(|v| v.batch_number == batch_number)
    }

    /// Get all votes since a batch number
    pub fn votes_since(&self, batch_number: BatchNumber) -> Vec<&MerkleVote> {
        self.votes
            .iter()
            .filter(|v| v.batch_number >= batch_number)
            .collect()
    }
}

impl Default for TowerVoteState {
    fn default() -> Self {
        Self::new()
    }
}

/// Lockout period for a vote (exponential backoff)
#[derive(Clone, Debug)]
pub struct Lockout {
    pub batch_number: BatchNumber,
    pub confirmation_count: u32,
}

impl Lockout {
    pub fn new(batch_number: BatchNumber) -> Self {
        Self {
            batch_number,
            confirmation_count: 1,
        }
    }

    /// Calculate lockout distance (exponential: 2^confirmation_count)
    pub fn lockout_distance(&self) -> u64 {
        2u64.pow(self.confirmation_count)
    }

    /// Check if this lockout expires at the given batch
    pub fn is_locked_out_at(&self, batch_number: BatchNumber) -> bool {
        batch_number < self.batch_number + self.lockout_distance()
    }
}

/// Tower BFT state for oracle consensus
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OracleTower {
    /// Node's public key
    pub node_pubkey: [u8; 32],
    
    /// Vote state
    pub vote_state: TowerVoteState,
    
    /// Threshold depth for switching forks
    pub threshold_depth: usize,
    
    /// Threshold size (stake percentage)
    pub threshold_size: f64,
    
    /// Vote history for replay protection
    #[serde(skip)]
    pub vote_history: HashMap<BatchNumber, MerkleRoot>,
    
    /// Lockouts for safety
    #[serde(skip)]
    pub lockouts: Vec<Lockout>,
}

impl OracleTower {
    pub fn new(node_pubkey: [u8; 32]) -> Self {
        Self {
            node_pubkey,
            vote_state: TowerVoteState::new(),
            threshold_depth: 8,
            threshold_size: 0.67, // 2/3
            vote_history: HashMap::new(),
            lockouts: Vec::new(),
        }
    }

    /// Check if we can vote for a batch (replay protection)
    pub fn can_vote(&self, batch_number: BatchNumber, root: &MerkleRoot) -> bool {
        // Check if we've already voted for this batch
        if let Some(existing_root) = self.vote_history.get(&batch_number) {
            // Can only vote again if it's the same root
            return existing_root == root;
        }

        // Check lockouts
        for lockout in &self.lockouts {
            if lockout.is_locked_out_at(batch_number) {
                return false;
            }
        }

        true
    }

    /// Record a vote
    pub fn record_vote(&mut self, batch_number: BatchNumber, root: MerkleRoot, timestamp: i64) -> Result<()> {
        // Check if we can vote
        if !self.can_vote(batch_number, &root) {
            return Err(anyhow::anyhow!("Cannot vote: replay protection or lockout"));
        }

        // Create vote
        let vote = MerkleVote {
            batch_number,
            root,
            timestamp,
        };

        // Update vote state
        self.vote_state.push_vote(vote);

        // Record in history
        self.vote_history.insert(batch_number, root);

        // Update lockouts
        self.update_lockouts(batch_number);

        Ok(())
    }

    /// Update lockouts after a vote
    fn update_lockouts(&mut self, batch_number: BatchNumber) {
        // Add new lockout
        self.lockouts.push(Lockout::new(batch_number));

        // Increment confirmation counts for previous lockouts
        for lockout in &mut self.lockouts {
            if lockout.batch_number < batch_number {
                lockout.confirmation_count += 1;
            }
        }

        // Remove expired lockouts (keep last 32)
        if self.lockouts.len() > 32 {
            self.lockouts.drain(0..self.lockouts.len() - 32);
        }
    }

    /// Check if a batch is locked out
    pub fn is_locked_out(&self, batch_number: BatchNumber) -> bool {
        self.lockouts.iter().any(|l| l.is_locked_out_at(batch_number))
    }

    /// Get the current root batch
    pub fn root_batch(&self) -> Option<BatchNumber> {
        self.vote_state.root_batch
    }

    /// Update root batch (confirmed)
    pub fn update_root(&mut self, batch_number: BatchNumber) {
        self.vote_state.root_batch = Some(batch_number);

        // Clean up old lockouts
        self.lockouts.retain(|l| l.batch_number >= batch_number);

        // Clean up old vote history
        self.vote_history.retain(|&b, _| b >= batch_number);
    }

    /// Check if we should switch to a different fork
    pub fn should_switch_fork(
        &self,
        _current_batch: BatchNumber,
        current_stake: u64,
        new_batch: BatchNumber,
        new_stake: u64,
        total_stake: u64,
    ) -> bool {
        // Can't switch if locked out
        if self.is_locked_out(new_batch) {
            return false;
        }

        // Calculate stake percentages
        let current_pct = (current_stake as f64) / (total_stake as f64);
        let new_pct = (new_stake as f64) / (total_stake as f64);

        // Switch if new fork has significantly more stake
        let threshold = self.threshold_size;
        new_pct > current_pct + threshold
    }

    /// Get vote statistics
    pub fn stats(&self) -> TowerStats {
        TowerStats {
            total_votes: self.vote_state.votes.len(),
            active_lockouts: self.lockouts.len(),
            root_batch: self.vote_state.root_batch,
            last_voted_batch: self.vote_state.last_voted_batch,
        }
    }
}

/// Tower statistics
#[derive(Clone, Debug)]
pub struct TowerStats {
    pub total_votes: usize,
    pub active_lockouts: usize,
    pub root_batch: Option<BatchNumber>,
    pub last_voted_batch: Option<BatchNumber>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tower_creation() {
        let pubkey = [1u8; 32];
        let tower = OracleTower::new(pubkey);
        assert_eq!(tower.node_pubkey, pubkey);
        assert_eq!(tower.vote_state.votes.len(), 0);
    }

    #[test]
    fn test_record_vote() {
        let pubkey = [1u8; 32];
        let mut tower = OracleTower::new(pubkey);

        let root = [42u8; 32];
        tower.record_vote(1, root, 1000).unwrap();

        assert_eq!(tower.vote_state.votes.len(), 1);
        assert_eq!(tower.vote_state.last_voted_batch, Some(1));
    }

    #[test]
    fn test_replay_protection() {
        let pubkey = [1u8; 32];
        let mut tower = OracleTower::new(pubkey);

        let root1 = [42u8; 32];
        let root2 = [43u8; 32];

        // First vote succeeds
        tower.record_vote(1, root1, 1000).unwrap();

        // Can vote again with same root
        assert!(tower.can_vote(1, &root1));

        // Cannot vote with different root (replay protection)
        assert!(!tower.can_vote(1, &root2));
    }

    #[test]
    fn test_lockouts() {
        let pubkey = [1u8; 32];
        let mut tower = OracleTower::new(pubkey);

        let root = [42u8; 32];
        tower.record_vote(10, root, 1000).unwrap();

        // Batch 10 creates lockout for batches 10-11 (2^1 = 2)
        assert!(tower.is_locked_out(10));
        assert!(tower.is_locked_out(11));
        assert!(!tower.is_locked_out(12));
    }

    #[test]
    fn test_exponential_lockout() {
        let lockout = Lockout::new(10);
        assert_eq!(lockout.lockout_distance(), 2); // 2^1

        let mut lockout2 = Lockout::new(10);
        lockout2.confirmation_count = 3;
        assert_eq!(lockout2.lockout_distance(), 8); // 2^3
    }

    #[test]
    fn test_update_root() {
        let pubkey = [1u8; 32];
        let mut tower = OracleTower::new(pubkey);

        let root = [42u8; 32];
        tower.record_vote(10, root, 1000).unwrap();
        tower.record_vote(11, root, 1001).unwrap();

        tower.update_root(10);
        assert_eq!(tower.root_batch(), Some(10));

        // Old votes should be cleaned up
        assert!(!tower.vote_history.contains_key(&9));
    }
}

