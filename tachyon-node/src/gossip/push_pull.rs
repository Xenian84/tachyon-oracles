#![allow(dead_code)]
/// Push/Pull Gossip Protocol
/// 
/// Inspired by Solana's gossip push/pull mechanism.
/// - Push: Broadcast new data to random peers
/// - Pull: Request missing data from peers

use super::crds::{Crds, VersionedCrdsValue, CrdsLabel};
use std::collections::HashSet;
use std::net::SocketAddr;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Push new values to peers
    Push(Vec<VersionedCrdsValue>),
    /// Pull request with bloom filter
    PullRequest {
        filter: BloomFilter,
        from: SocketAddr,
    },
    /// Pull response with values
    PullResponse(Vec<VersionedCrdsValue>),
    /// Ping/Pong for liveness
    Ping(u64),
    Pong(u64),
}

/// Simple bloom filter for pull requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilter {
    /// Set of labels we already have
    pub known_labels: HashSet<String>,
}

impl BloomFilter {
    pub fn new() -> Self {
        Self {
            known_labels: HashSet::new(),
        }
    }

    pub fn add(&mut self, label: &CrdsLabel) {
        self.known_labels.insert(format!("{:?}", label));
    }

    pub fn contains(&self, label: &CrdsLabel) -> bool {
        self.known_labels.contains(&format!("{:?}", label))
    }
}

impl Default for BloomFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Push gossip manager
pub struct PushGossip {
    /// Number of peers to push to
    fanout: usize,
}

impl PushGossip {
    pub fn new(fanout: usize) -> Self {
        Self { fanout }
    }

    /// Select random peers for push
    pub fn select_peers<'a>(
        &self,
        peers: &'a [SocketAddr],
        exclude: Option<&SocketAddr>,
    ) -> Vec<&'a SocketAddr> {
        let mut rng = rand::thread_rng();
        let mut available: Vec<_> = peers
            .iter()
            .filter(|p| Some(*p) != exclude)
            .collect();
        
        available.shuffle(&mut rng);
        available.into_iter().take(self.fanout).collect()
    }

    /// Create push message with recent updates
    pub fn create_push_message(&self, crds: &Crds, since: u64) -> GossipMessage {
        let values: Vec<_> = crds
            .values()
            .filter(|v| v.wallclock > since)
            .cloned()
            .collect();
        
        GossipMessage::Push(values)
    }
}

/// Pull gossip manager
pub struct PullGossip {
    /// Minimum time between pull requests (ms)
    pull_interval_ms: u64,
    /// Last pull timestamp
    last_pull: u64,
}

impl PullGossip {
    pub fn new(pull_interval_ms: u64) -> Self {
        Self {
            pull_interval_ms,
            last_pull: 0,
        }
    }

    /// Check if it's time to pull
    pub fn should_pull(&self, now: u64) -> bool {
        now - self.last_pull >= self.pull_interval_ms
    }

    /// Create pull request with bloom filter
    pub fn create_pull_request(
        &mut self,
        crds: &Crds,
        from: SocketAddr,
        now: u64,
    ) -> GossipMessage {
        self.last_pull = now;
        
        // Build bloom filter of what we have
        let mut filter = BloomFilter::new();
        for value in crds.values() {
            filter.add(&value.value.label());
        }
        
        GossipMessage::PullRequest { filter, from }
    }

    /// Process pull request and create response
    pub fn process_pull_request(
        &self,
        crds: &Crds,
        filter: &BloomFilter,
    ) -> GossipMessage {
        // Send values that the requester doesn't have
        let values: Vec<_> = crds
            .values()
            .filter(|v| !filter.contains(&v.value.label()))
            .cloned()
            .collect();
        
        GossipMessage::PullResponse(values)
    }

    /// Process pull response and insert into CRDS
    pub fn process_pull_response(
        &self,
        crds: &mut Crds,
        values: Vec<VersionedCrdsValue>,
    ) -> usize {
        let mut inserted = 0;
        for value in values {
            if crds.insert(value).is_ok() {
                inserted += 1;
            }
        }
        inserted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::crds::{CrdsValue, ContactInfo};
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_push_select_peers() {
        let push = PushGossip::new(3);
        let peers: Vec<SocketAddr> = vec![
            "127.0.0.1:7777".parse().unwrap(),
            "127.0.0.1:7778".parse().unwrap(),
            "127.0.0.1:7779".parse().unwrap(),
            "127.0.0.1:7780".parse().unwrap(),
        ];
        
        let selected = push.select_peers(&peers, None);
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn test_bloom_filter() {
        let mut filter = BloomFilter::new();
        let pubkey = Pubkey::new_unique();
        let label = CrdsLabel::ContactInfo(pubkey);
        
        assert!(!filter.contains(&label));
        filter.add(&label);
        assert!(filter.contains(&label));
    }

    #[test]
    fn test_pull_request_response() {
        let mut crds = Crds::new(1000);
        let pubkey = Pubkey::new_unique();
        
        let contact = ContactInfo {
            pubkey,
            gossip_addr: "127.0.0.1:7777".parse().unwrap(),
            api_addr: "127.0.0.1:8080".parse().unwrap(),
            version: 1,
        };
        
        let value = VersionedCrdsValue {
            value: CrdsValue::ContactInfo(contact),
            wallclock: 100,
            signature: [0; 64],
        };
        
        crds.insert(value).unwrap();
        
        let pull = PullGossip::new(5000);
        let filter = BloomFilter::new(); // Empty filter
        
        let response = pull.process_pull_request(&crds, &filter);
        
        match response {
            GossipMessage::PullResponse(values) => {
                assert_eq!(values.len(), 1);
            }
            _ => panic!("Expected PullResponse"),
        }
    }
}

