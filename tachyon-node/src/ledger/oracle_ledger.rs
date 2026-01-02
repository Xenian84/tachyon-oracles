#![allow(dead_code)]
// Oracle Ledger - Historical price data storage
// Simplified from Solana Ledger for Tachyon Oracle Network

use std::sync::Arc;
use rocksdb::{DB, Options, IteratorMode};
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Price entry for historical storage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceRecord {
    pub symbol: String,
    pub price: f64,
    pub timestamp: i64,
    pub batch_number: u64,
    pub merkle_root: [u8; 32],
    pub submitter: [u8; 32],
}

/// Merkle root record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleRootRecord {
    pub root: [u8; 32],
    pub batch_number: u64,
    pub feed_count: u32,
    pub timestamp: i64,
    pub submitter: [u8; 32],
}

/// Historical ledger for price data
pub struct OracleLedger {
    db: Arc<DB>,
}

impl OracleLedger {
    pub fn new(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(1000);
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(128 * 1024 * 1024); // 128MB
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Store a price record
    pub fn store_price(&self, record: &PriceRecord) -> Result<()> {
        // Key: symbol:timestamp
        let key = format!("price:{}:{}", record.symbol, record.timestamp);
        let data = bincode::serialize(record)?;
        self.db.put(key.as_bytes(), &data)?;
        Ok(())
    }

    /// Store a Merkle root record
    pub fn store_merkle_root(&self, record: &MerkleRootRecord) -> Result<()> {
        // Key: root:batch_number
        let key = format!("root:{}", record.batch_number);
        let data = bincode::serialize(record)?;
        self.db.put(key.as_bytes(), &data)?;
        Ok(())
    }

    /// Get price history for a symbol
    pub fn get_price_history(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<PriceRecord>> {
        let mut records = Vec::new();
        
        let prefix = format!("price:{}:", symbol);
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        
        for item in iter {
            let (_key, value) = item?;
            let record: PriceRecord = bincode::deserialize(&value)?;
            
            if record.timestamp >= start_time && record.timestamp <= end_time {
                records.push(record.clone());
            }
            
            // Stop if we've passed the end time
            if record.timestamp > end_time {
                break;
            }
        }
        
        Ok(records)
    }

    /// Get latest price for a symbol
    pub fn get_latest_price(&self, symbol: &str) -> Result<Option<PriceRecord>> {
        let prefix = format!("price:{}:", symbol);
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        
        let mut latest: Option<PriceRecord> = None;
        
        for item in iter {
            let (_key, value) = item?;
            let record: PriceRecord = bincode::deserialize(&value)?;
            
            let should_update = if let Some(ref current_latest) = latest {
                record.timestamp > current_latest.timestamp
            } else {
                true
            };
            
            if should_update {
                latest = Some(record);
            }
        }
        
        Ok(latest)
    }

    /// Get Merkle root by batch number
    pub fn get_merkle_root(&self, batch_number: u64) -> Result<Option<MerkleRootRecord>> {
        let key = format!("root:{}", batch_number);
        
        if let Some(data) = self.db.get(key.as_bytes())? {
            let record: MerkleRootRecord = bincode::deserialize(&data)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Get all Merkle roots in a range
    pub fn get_merkle_roots_range(
        &self,
        start_batch: u64,
        end_batch: u64,
    ) -> Result<Vec<MerkleRootRecord>> {
        let mut records = Vec::new();
        
        for batch in start_batch..=end_batch {
            if let Some(record) = self.get_merkle_root(batch)? {
                records.push(record);
            }
        }
        
        Ok(records)
    }

    /// Get all symbols with price data
    pub fn get_symbols(&self) -> Result<Vec<String>> {
        let mut symbols = std::collections::HashSet::new();
        
        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            let (key, _value) = item?;
            let key_str = String::from_utf8_lossy(&key);
            
            if key_str.starts_with("price:") {
                // Extract symbol from key "price:SYMBOL:timestamp"
                if let Some(symbol) = key_str.split(':').nth(1) {
                    symbols.insert(symbol.to_string());
                }
            }
        }
        
        Ok(symbols.into_iter().collect())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<LedgerStats> {
        let mut price_count = 0u64;
        let mut root_count = 0u64;
        let mut total_size = 0u64;
        
        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);
            
            if key_str.starts_with("price:") {
                price_count += 1;
            } else if key_str.starts_with("root:") {
                root_count += 1;
            }
            
            total_size += key.len() as u64 + value.len() as u64;
        }
        
        Ok(LedgerStats {
            price_count,
            root_count,
            total_size,
        })
    }

    /// Compact the database
    pub fn compact(&self) -> Result<()> {
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        Ok(())
    }

    /// Flush to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Delete old price data (cleanup)
    pub fn delete_old_prices(&self, before_timestamp: i64) -> Result<u64> {
        let mut deleted = 0u64;
        let mut keys_to_delete = Vec::new();
        
        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);
            
            if key_str.starts_with("price:") {
                let record: PriceRecord = bincode::deserialize(&value)?;
                if record.timestamp < before_timestamp {
                    keys_to_delete.push(key.to_vec());
                }
            }
        }
        
        for key in keys_to_delete {
            self.db.delete(&key)?;
            deleted += 1;
        }
        
        Ok(deleted)
    }
}

/// Ledger statistics
#[derive(Debug, Clone)]
pub struct LedgerStats {
    pub price_count: u64,
    pub root_count: u64,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_store_and_retrieve_price() {
        let temp_dir = TempDir::new().unwrap();
        let ledger = OracleLedger::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let record = PriceRecord {
            symbol: "BTC/USD".to_string(),
            price: 50000.0,
            timestamp: 1000,
            batch_number: 1,
            merkle_root: [1u8; 32],
            submitter: [0u8; 32],
        };
        
        ledger.store_price(&record).unwrap();
        
        let history = ledger.get_price_history("BTC/USD", 0, 2000).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].price, 50000.0);
    }

    #[test]
    fn test_store_and_retrieve_merkle_root() {
        let temp_dir = TempDir::new().unwrap();
        let ledger = OracleLedger::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let record = MerkleRootRecord {
            root: [42u8; 32],
            batch_number: 1,
            feed_count: 10,
            timestamp: 1000,
            submitter: [0u8; 32],
        };
        
        ledger.store_merkle_root(&record).unwrap();
        
        let retrieved = ledger.get_merkle_root(1).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().feed_count, 10);
    }

    #[test]
    fn test_get_symbols() {
        let temp_dir = TempDir::new().unwrap();
        let ledger = OracleLedger::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let symbols = vec!["BTC/USD", "ETH/USD", "SOL/USD"];
        
        for (i, symbol) in symbols.iter().enumerate() {
            let record = PriceRecord {
                symbol: symbol.to_string(),
                price: 1000.0 * (i as f64 + 1.0),
                timestamp: 1000 + i as i64,
                batch_number: i as u64,
                merkle_root: [i as u8; 32],
                submitter: [0u8; 32],
            };
            ledger.store_price(&record).unwrap();
        }
        
        let retrieved_symbols = ledger.get_symbols().unwrap();
        assert_eq!(retrieved_symbols.len(), 3);
    }

    #[test]
    fn test_ledger_stats() {
        let temp_dir = TempDir::new().unwrap();
        let ledger = OracleLedger::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        // Store some data
        for i in 0..10 {
            let record = PriceRecord {
                symbol: "BTC/USD".to_string(),
                price: 50000.0 + i as f64,
                timestamp: 1000 + i,
                batch_number: i as u64,
                merkle_root: [i as u8; 32],
                submitter: [0u8; 32],
            };
            ledger.store_price(&record).unwrap();
        }
        
        let stats = ledger.get_stats().unwrap();
        assert_eq!(stats.price_count, 10);
    }
}

