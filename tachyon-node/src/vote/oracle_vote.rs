#![allow(dead_code)]
// Oracle Vote - Validator voting for consensus
// Simplified voting mechanism for Tachyon Oracle Network

use std::collections::HashMap;

/// Vote for a Merkle root
#[derive(Clone, Debug)]
pub struct MerkleRootVote {
    pub root: [u8; 32],
    pub batch_number: u64,
    pub voter: [u8; 32], // Validator pubkey
    pub stake: u64,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

/// Vote state for a validator
#[derive(Clone, Debug)]
pub struct VoteState {
    pub validator: [u8; 32],
    pub stake: u64,
    pub votes: Vec<MerkleRootVote>,
    pub last_vote_timestamp: i64,
}

impl VoteState {
    pub fn new(validator: [u8; 32], stake: u64) -> Self {
        Self {
            validator,
            stake,
            votes: Vec::new(),
            last_vote_timestamp: 0,
        }
    }

    /// Add a vote
    pub fn add_vote(&mut self, vote: MerkleRootVote) {
        self.last_vote_timestamp = vote.timestamp;
        self.votes.push(vote);
    }

    /// Get the latest vote
    pub fn latest_vote(&self) -> Option<&MerkleRootVote> {
        self.votes.last()
    }

    /// Check if voted for a specific root
    pub fn has_voted_for(&self, root: &[u8; 32]) -> bool {
        self.votes.iter().any(|v| &v.root == root)
    }
}

/// Vote tracker for consensus
pub struct VoteTracker {
    votes: HashMap<[u8; 32], Vec<MerkleRootVote>>, // root -> votes
    vote_states: HashMap<[u8; 32], VoteState>,     // validator -> state
    total_stake: u64,
}

impl VoteTracker {
    pub fn new() -> Self {
        Self {
            votes: HashMap::new(),
            vote_states: HashMap::new(),
            total_stake: 0,
        }
    }

    /// Register a validator
    pub fn register_validator(&mut self, validator: [u8; 32], stake: u64) {
        self.vote_states
            .insert(validator, VoteState::new(validator, stake));
        self.total_stake += stake;
    }

    /// Update validator stake
    pub fn update_stake(&mut self, validator: [u8; 32], new_stake: u64) {
        if let Some(state) = self.vote_states.get_mut(&validator) {
            self.total_stake = self.total_stake - state.stake + new_stake;
            state.stake = new_stake;
        }
    }

    /// Record a vote
    pub fn record_vote(&mut self, vote: MerkleRootVote) {
        // Add to root votes
        self.votes.entry(vote.root).or_insert_with(Vec::new).push(vote.clone());

        // Update vote state
        if let Some(state) = self.vote_states.get_mut(&vote.voter) {
            state.add_vote(vote);
        }
    }

    /// Get votes for a specific root
    pub fn get_votes_for_root(&self, root: &[u8; 32]) -> Option<&Vec<MerkleRootVote>> {
        self.votes.get(root)
    }

    /// Calculate stake weight for a root
    pub fn get_stake_weight(&self, root: &[u8; 32]) -> u64 {
        self.votes
            .get(root)
            .map(|votes| votes.iter().map(|v| v.stake).sum())
            .unwrap_or(0)
    }

    /// Check if root has reached consensus (2/3+ stake)
    pub fn has_consensus(&self, root: &[u8; 32]) -> bool {
        if self.total_stake == 0 {
            return false;
        }

        let stake_weight = self.get_stake_weight(root);
        let required_stake = (self.total_stake * 2) / 3;

        stake_weight >= required_stake
    }

    /// Get all roots that have reached consensus
    pub fn get_consensus_roots(&self) -> Vec<[u8; 32]> {
        self.votes
            .keys()
            .filter(|root| self.has_consensus(root))
            .copied()
            .collect()
    }

    /// Get vote participation rate
    pub fn get_participation_rate(&self, root: &[u8; 32]) -> f64 {
        if self.total_stake == 0 {
            return 0.0;
        }

        let stake_weight = self.get_stake_weight(root);
        (stake_weight as f64) / (self.total_stake as f64)
    }

    /// Get total stake
    pub fn total_stake(&self) -> u64 {
        self.total_stake
    }

    /// Get number of validators
    pub fn num_validators(&self) -> usize {
        self.vote_states.len()
    }
}

impl Default for VoteTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_state() {
        let validator = [1u8; 32];
        let mut state = VoteState::new(validator, 1000);

        let vote = MerkleRootVote {
            root: [42u8; 32],
            batch_number: 1,
            voter: validator,
            stake: 1000,
            timestamp: 1000,
            signature: vec![],
        };

        state.add_vote(vote.clone());
        assert_eq!(state.votes.len(), 1);
        assert!(state.has_voted_for(&[42u8; 32]));
    }

    #[test]
    fn test_vote_tracker() {
        let mut tracker = VoteTracker::new();

        // Register validators
        let val1 = [1u8; 32];
        let val2 = [2u8; 32];
        let val3 = [3u8; 32];

        tracker.register_validator(val1, 1000);
        tracker.register_validator(val2, 1000);
        tracker.register_validator(val3, 1000);

        assert_eq!(tracker.total_stake(), 3000);
        assert_eq!(tracker.num_validators(), 3);
    }

    #[test]
    fn test_consensus() {
        let mut tracker = VoteTracker::new();

        // Register 3 validators with equal stake
        let val1 = [1u8; 32];
        let val2 = [2u8; 32];
        let val3 = [3u8; 32];

        tracker.register_validator(val1, 1000);
        tracker.register_validator(val2, 1000);
        tracker.register_validator(val3, 1000);

        let root = [42u8; 32];

        // No consensus yet
        assert!(!tracker.has_consensus(&root));

        // Val1 votes
        tracker.record_vote(MerkleRootVote {
            root,
            batch_number: 1,
            voter: val1,
            stake: 1000,
            timestamp: 1000,
            signature: vec![],
        });

        // Still no consensus (1/3 stake)
        assert!(!tracker.has_consensus(&root));

        // Val2 votes
        tracker.record_vote(MerkleRootVote {
            root,
            batch_number: 1,
            voter: val2,
            stake: 1000,
            timestamp: 1001,
            signature: vec![],
        });

        // Now we have consensus (2/3 stake)
        assert!(tracker.has_consensus(&root));
    }

    #[test]
    fn test_participation_rate() {
        let mut tracker = VoteTracker::new();

        tracker.register_validator([1u8; 32], 1000);
        tracker.register_validator([2u8; 32], 1000);

        let root = [42u8; 32];

        tracker.record_vote(MerkleRootVote {
            root,
            batch_number: 1,
            voter: [1u8; 32],
            stake: 1000,
            timestamp: 1000,
            signature: vec![],
        });

        assert_eq!(tracker.get_participation_rate(&root), 0.5);
    }
}

