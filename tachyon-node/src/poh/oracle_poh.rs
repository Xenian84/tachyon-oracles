#![allow(dead_code)]
// Oracle PoH (Proof of History) - Adapted from Solana for Tachyon Oracle Network
// Deterministic ordering of price submissions using SHA256 hashing

use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

/// PoH entry for price submissions
#[derive(Clone, Debug)]
pub struct PriceEntry {
    pub hash: [u8; 32],
    pub num_hashes: u64,
    pub timestamp: i64,
    pub price_data: Option<Vec<u8>>,
}

/// PoH recorder for deterministic ordering
pub struct PohRecorder {
    current_hash: [u8; 32],
    num_hashes: u64,
    hashes_per_tick: u64,
}

impl PohRecorder {
    pub fn new(seed: [u8; 32], hashes_per_tick: u64) -> Self {
        Self {
            current_hash: seed,
            num_hashes: 0,
            hashes_per_tick,
        }
    }

    /// Hash the current hash to advance PoH
    pub fn hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.current_hash);
        self.current_hash = hasher.finalize().into();
        self.num_hashes += 1;
    }

    /// Hash multiple times
    pub fn hash_n(&mut self, n: u64) {
        for _ in 0..n {
            self.hash();
        }
    }

    /// Record a price entry with the current PoH state
    pub fn record(&mut self, price_data: Vec<u8>) -> PriceEntry {
        // Hash the price data into the PoH sequence
        let mut hasher = Sha256::new();
        hasher.update(&self.current_hash);
        hasher.update(&price_data);
        self.current_hash = hasher.finalize().into();
        self.num_hashes += 1;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        PriceEntry {
            hash: self.current_hash,
            num_hashes: self.num_hashes,
            timestamp,
            price_data: Some(price_data),
        }
    }

    /// Create a tick (periodic marker in PoH)
    pub fn tick(&mut self) -> PriceEntry {
        // Hash until we reach the next tick
        let hashes_until_tick = self.hashes_per_tick - (self.num_hashes % self.hashes_per_tick);
        self.hash_n(hashes_until_tick);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        PriceEntry {
            hash: self.current_hash,
            num_hashes: self.num_hashes,
            timestamp,
            price_data: None, // Ticks have no data
        }
    }

    /// Get the current PoH hash
    pub fn current_hash(&self) -> [u8; 32] {
        self.current_hash
    }

    /// Get the current number of hashes
    pub fn num_hashes(&self) -> u64 {
        self.num_hashes
    }

    /// Verify a PoH entry
    pub fn verify_entry(&self, entry: &PriceEntry, previous_hash: &[u8; 32]) -> bool {
        let mut hash = *previous_hash;
        
        // If there's price data, hash it
        if let Some(data) = &entry.price_data {
            let mut hasher = Sha256::new();
            hasher.update(&hash);
            hasher.update(data);
            hash = hasher.finalize().into();
        } else {
            // It's a tick, hash multiple times
            for _ in 0..self.hashes_per_tick {
                let mut hasher = Sha256::new();
                hasher.update(&hash);
                hash = hasher.finalize().into();
            }
        }
        
        hash == entry.hash
    }
}

/// PoH service for continuous hashing
pub struct PohService {
    recorder: PohRecorder,
    running: bool,
}

impl PohService {
    pub fn new(seed: [u8; 32], hashes_per_tick: u64) -> Self {
        Self {
            recorder: PohRecorder::new(seed, hashes_per_tick),
            running: false,
        }
    }

    /// Start the PoH service
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the PoH service
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Record a price submission
    pub fn record_price(&mut self, price_data: Vec<u8>) -> PriceEntry {
        self.recorder.record(price_data)
    }

    /// Generate a tick
    pub fn generate_tick(&mut self) -> PriceEntry {
        self.recorder.tick()
    }

    /// Get current PoH state
    pub fn current_state(&self) -> ([u8; 32], u64) {
        (self.recorder.current_hash(), self.recorder.num_hashes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poh_recorder() {
        let seed = [0u8; 32];
        let mut recorder = PohRecorder::new(seed, 100);
        
        // Hash once
        let initial_hash = recorder.current_hash();
        recorder.hash();
        assert_ne!(recorder.current_hash(), initial_hash);
        assert_eq!(recorder.num_hashes(), 1);
    }

    #[test]
    fn test_record_price() {
        let seed = [0u8; 32];
        let mut recorder = PohRecorder::new(seed, 100);
        
        let price_data = vec![1, 2, 3, 4];
        let entry = recorder.record(price_data);
        
        assert!(entry.price_data.is_some());
        assert_eq!(entry.num_hashes, 1);
    }

    #[test]
    fn test_tick() {
        let seed = [0u8; 32];
        let mut recorder = PohRecorder::new(seed, 10);
        
        let entry = recorder.tick();
        assert!(entry.price_data.is_none());
        assert_eq!(entry.num_hashes, 10);
    }

    #[test]
    fn test_verify_entry() {
        let seed = [0u8; 32];
        let mut recorder = PohRecorder::new(seed, 100);
        
        let previous_hash = recorder.current_hash();
        let price_data = vec![1, 2, 3, 4];
        let entry = recorder.record(price_data);
        
        assert!(recorder.verify_entry(&entry, &previous_hash));
    }

    #[test]
    fn test_poh_service() {
        let seed = [0u8; 32];
        let mut service = PohService::new(seed, 100);
        
        assert!(!service.is_running());
        service.start();
        assert!(service.is_running());
        
        let price_data = vec![1, 2, 3, 4];
        let entry = service.record_price(price_data);
        assert!(entry.price_data.is_some());
    }
}

