#![allow(dead_code)]
/// CRDS (Conflict-free Replicated Data Store)
/// 
/// Inspired by Solana's gossip CRDS implementation.
/// Stores versioned oracle data with conflict resolution.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Versioned CRDS value with timestamp and signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedCrdsValue {
    pub value: CrdsValue,
    pub wallclock: u64,
    pub signature: Vec<u8>,
}

/// CRDS value types for oracle network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdsValue {
    /// Node contact information
    ContactInfo(ContactInfo),
    /// Price update from oracle
    PriceData(PriceData),
    /// Consensus vote
    Vote(Vote),
    /// Node stake information
    StakeInfo(StakeInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub pubkey: Pubkey,
    pub gossip_addr: std::net::SocketAddr,
    pub api_addr: std::net::SocketAddr,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub pubkey: Pubkey,
    pub asset: String,
    pub price: f64,
    pub confidence: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub pubkey: Pubkey,
    pub root: [u8; 32],
    pub stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub pubkey: Pubkey,
    pub stake: u64,
    pub is_active: bool,
}

/// CRDS label for indexing values
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum CrdsLabel {
    ContactInfo(Pubkey),
    PriceData(Pubkey, String), // pubkey + asset
    Vote(Pubkey, u64),          // pubkey + slot
    StakeInfo(Pubkey),
}

impl CrdsValue {
    pub fn label(&self) -> CrdsLabel {
        match self {
            CrdsValue::ContactInfo(info) => CrdsLabel::ContactInfo(info.pubkey),
            CrdsValue::PriceData(data) => CrdsLabel::PriceData(data.pubkey, data.asset.clone()),
            CrdsValue::Vote(vote) => CrdsLabel::Vote(vote.pubkey, 0), // slot would go here
            CrdsValue::StakeInfo(info) => CrdsLabel::StakeInfo(info.pubkey),
        }
    }

    pub fn pubkey(&self) -> Pubkey {
        match self {
            CrdsValue::ContactInfo(info) => info.pubkey,
            CrdsValue::PriceData(data) => data.pubkey,
            CrdsValue::Vote(vote) => vote.pubkey,
            CrdsValue::StakeInfo(info) => info.pubkey,
        }
    }
}

/// CRDS store with conflict resolution
pub struct Crds {
    table: HashMap<CrdsLabel, VersionedCrdsValue>,
    /// Maximum entries before pruning
    max_entries: usize,
}

impl Crds {
    pub fn new(max_entries: usize) -> Self {
        Self {
            table: HashMap::new(),
            max_entries,
        }
    }

    /// Insert or update a value with conflict resolution
    pub fn insert(&mut self, value: VersionedCrdsValue) -> Result<(), CrdsError> {
        let label = value.value.label();
        
        // Check if we should update
        if let Some(existing) = self.table.get(&label) {
            if !Self::should_override(existing, &value) {
                return Err(CrdsError::InsertFailed);
            }
        }
        
        self.table.insert(label, value);
        
        // Prune if needed
        if self.table.len() > self.max_entries {
            self.prune();
        }
        
        Ok(())
    }

    /// Get a value by label
    pub fn get(&self, label: &CrdsLabel) -> Option<&VersionedCrdsValue> {
        self.table.get(label)
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &VersionedCrdsValue> {
        self.table.values()
    }

    /// Get all values for a specific pubkey
    pub fn get_by_pubkey(&self, pubkey: &Pubkey) -> Vec<&VersionedCrdsValue> {
        self.table
            .values()
            .filter(|v| &v.value.pubkey() == pubkey)
            .collect()
    }

    /// Conflict resolution: newer wallclock wins
    fn should_override(existing: &VersionedCrdsValue, new: &VersionedCrdsValue) -> bool {
        new.wallclock > existing.wallclock
    }

    /// Prune old entries (keep most recent 80%)
    fn prune(&mut self) {
        let target_size = (self.max_entries * 80) / 100;
        
        // Sort by wallclock and keep newest
        let mut entries: Vec<_> = self.table.iter().map(|(k, v)| (k.clone(), v.wallclock)).collect();
        entries.sort_by_key(|(_, wallclock)| *wallclock);
        
        // Remove oldest entries
        let to_remove = entries.len().saturating_sub(target_size);
        for (label, _) in entries.iter().take(to_remove) {
            self.table.remove(label);
        }
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

#[derive(Debug)]
pub enum CrdsError {
    InsertFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crds_insert_and_get() {
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
            signature: vec![0; 64],
        };
        
        crds.insert(value.clone()).unwrap();
        
        let label = CrdsLabel::ContactInfo(pubkey);
        assert!(crds.get(&label).is_some());
    }

    #[test]
    fn test_crds_conflict_resolution() {
        let mut crds = Crds::new(1000);
        let pubkey = Pubkey::new_unique();
        
        let contact1 = ContactInfo {
            pubkey,
            gossip_addr: "127.0.0.1:7777".parse().unwrap(),
            api_addr: "127.0.0.1:8080".parse().unwrap(),
            version: 1,
        };
        
        let value1 = VersionedCrdsValue {
            value: CrdsValue::ContactInfo(contact1.clone()),
            wallclock: 100,
            signature: [0; 64],
        };
        
        let value2 = VersionedCrdsValue {
            value: CrdsValue::ContactInfo(contact1),
            wallclock: 200, // Newer
            signature: [0; 64],
        };
        
        crds.insert(value1).unwrap();
        crds.insert(value2.clone()).unwrap();
        
        let label = CrdsLabel::ContactInfo(pubkey);
        let stored = crds.get(&label).unwrap();
        assert_eq!(stored.wallclock, 200);
    }
}

