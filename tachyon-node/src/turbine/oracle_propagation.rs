#![allow(dead_code)]
// Oracle Propagation - Simplified from Solana Turbine for Tachyon Oracle Network
// Efficient Merkle root propagation using tree topology

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
};
use rand::{seq::SliceRandom, thread_rng};

/// Maximum fanout for data propagation
const FANOUT: usize = 200;

/// Maximum number of hops for propagation
pub const MAX_HOPS: usize = 4;

/// Merkle root message for propagation
#[derive(Clone, Debug)]
pub struct MerkleRootMessage {
    pub root: [u8; 32],
    pub batch_number: u64,
    pub feed_count: u32,
    pub timestamp: i64,
    pub submitter: [u8; 32], // Pubkey
    pub signature: Vec<u8>,
}

/// Oracle node in the propagation network
#[derive(Clone, Debug)]
pub struct OracleNode {
    pub pubkey: [u8; 32],
    pub addr: SocketAddr,
    pub stake: u64,
}

/// Propagation tree for efficient data distribution
pub struct PropagationTree {
    local_pubkey: [u8; 32],
    nodes: Vec<OracleNode>,
    stake_map: HashMap<[u8; 32], u64>,
}

impl PropagationTree {
    pub fn new(local_pubkey: [u8; 32]) -> Self {
        Self {
            local_pubkey,
            nodes: Vec::new(),
            stake_map: HashMap::new(),
        }
    }

    /// Add a node to the propagation tree
    pub fn add_node(&mut self, node: OracleNode) {
        self.stake_map.insert(node.pubkey, node.stake);
        self.nodes.push(node);
    }

    /// Update node stake
    pub fn update_stake(&mut self, pubkey: [u8; 32], stake: u64) {
        self.stake_map.insert(pubkey, stake);
        if let Some(node) = self.nodes.iter_mut().find(|n| n.pubkey == pubkey) {
            node.stake = stake;
        }
    }

    /// Get nodes to propagate to (stake-weighted selection)
    pub fn get_propagation_targets(&self, max_targets: usize) -> Vec<OracleNode> {
        let mut nodes = self.nodes.clone();
        
        // Sort by stake (descending)
        nodes.sort_by(|a, b| b.stake.cmp(&a.stake));
        
        // Take top stake-weighted nodes
        let mut targets: Vec<OracleNode> = nodes
            .into_iter()
            .filter(|n| n.pubkey != self.local_pubkey)
            .take(max_targets)
            .collect();
        
        // Shuffle for randomness (prevents hot spots)
        targets.shuffle(&mut thread_rng());
        
        targets
    }

    /// Calculate tree level for a node based on stake
    pub fn get_tree_level(&self, pubkey: &[u8; 32]) -> usize {
        let stake = self.stake_map.get(pubkey).copied().unwrap_or(0);
        let total_stake: u64 = self.stake_map.values().sum();
        
        if total_stake == 0 {
            return MAX_HOPS;
        }
        
        // Higher stake = lower level (closer to root)
        let stake_ratio = (stake as f64) / (total_stake as f64);
        
        if stake_ratio > 0.1 {
            0 // Top 10% stake - level 0
        } else if stake_ratio > 0.01 {
            1 // Top 1% stake - level 1
        } else if stake_ratio > 0.001 {
            2 // Top 0.1% stake - level 2
        } else {
            3 // Everyone else - level 3
        }
    }

    /// Get children nodes for propagation (tree topology)
    pub fn get_children(&self, level: usize) -> Vec<OracleNode> {
        if level >= MAX_HOPS {
            return Vec::new();
        }
        
        let fanout = std::cmp::min(FANOUT, self.nodes.len());
        self.get_propagation_targets(fanout)
    }
}

/// Propagation manager for Merkle roots
pub struct PropagationManager {
    tree: Arc<PropagationTree>,
    seen_roots: HashSet<[u8; 32]>,
}

impl PropagationManager {
    pub fn new(local_pubkey: [u8; 32]) -> Self {
        Self {
            tree: Arc::new(PropagationTree::new(local_pubkey)),
            seen_roots: HashSet::new(),
        }
    }

    /// Check if we've already seen this root
    pub fn has_seen(&self, root: &[u8; 32]) -> bool {
        self.seen_roots.contains(root)
    }

    /// Mark root as seen
    pub fn mark_seen(&mut self, root: [u8; 32]) {
        self.seen_roots.insert(root);
    }

    /// Propagate a Merkle root to the network
    pub fn propagate(&mut self, message: &MerkleRootMessage) -> Vec<(SocketAddr, MerkleRootMessage)> {
        // Check if already seen
        if self.has_seen(&message.root) {
            return Vec::new();
        }
        
        // Mark as seen
        self.mark_seen(message.root);
        
        // Get propagation targets
        let targets = self.tree.get_propagation_targets(FANOUT);
        
        // Create messages for each target
        targets
            .into_iter()
            .map(|node| (node.addr, message.clone()))
            .collect()
    }

    /// Update the propagation tree
    pub fn update_tree(&mut self, tree: PropagationTree) {
        self.tree = Arc::new(tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_tree() {
        let local_pubkey = [1u8; 32];
        let mut tree = PropagationTree::new(local_pubkey);
        
        let node = OracleNode {
            pubkey: [2u8; 32],
            addr: "127.0.0.1:8000".parse().unwrap(),
            stake: 1000,
        };
        
        tree.add_node(node);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn test_stake_weighted_selection() {
        let local_pubkey = [1u8; 32];
        let mut tree = PropagationTree::new(local_pubkey);
        
        // Add nodes with different stakes
        for i in 0..10 {
            let node = OracleNode {
                pubkey: [i as u8; 32],
                addr: format!("127.0.0.1:800{}", i).parse().unwrap(),
                stake: (i as u64 + 1) * 100,
            };
            tree.add_node(node);
        }
        
        let targets = tree.get_propagation_targets(5);
        assert!(targets.len() <= 5);
    }

    #[test]
    fn test_propagation_manager() {
        let local_pubkey = [1u8; 32];
        let mut manager = PropagationManager::new(local_pubkey);
        
        let root = [42u8; 32];
        assert!(!manager.has_seen(&root));
        
        manager.mark_seen(root);
        assert!(manager.has_seen(&root));
    }
}

