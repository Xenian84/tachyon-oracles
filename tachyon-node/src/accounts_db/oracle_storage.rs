#![allow(dead_code)]
// Oracle Storage - High-performance state storage
// Simplified from Solana Accounts-DB for Tachyon Oracle Network

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rocksdb::{DB, Options, WriteBatch};
use anyhow::Result;

/// Oracle account (validator state, staker info, etc.)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OracleAccount {
    pub pubkey: [u8; 32],
    pub data: Vec<u8>,
    pub lamports: u64,
    pub owner: [u8; 32],
    pub last_updated: i64,
}

/// In-memory cache for hot accounts
pub struct AccountCache {
    cache: Arc<RwLock<HashMap<[u8; 32], OracleAccount>>>,
    max_size: usize,
}

impl AccountCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    pub fn get(&self, pubkey: &[u8; 32]) -> Option<OracleAccount> {
        self.cache.read().unwrap().get(pubkey).cloned()
    }

    pub fn insert(&self, account: OracleAccount) {
        let mut cache = self.cache.write().unwrap();
        
        // Simple eviction if cache is full
        if cache.len() >= self.max_size {
            if let Some(key) = cache.keys().next().copied() {
                cache.remove(&key);
            }
        }
        
        cache.insert(account.pubkey, account);
    }

    pub fn remove(&self, pubkey: &[u8; 32]) {
        self.cache.write().unwrap().remove(pubkey);
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.read().unwrap().is_empty()
    }
}

/// Main accounts database
pub struct AccountsDb {
    db: Arc<DB>,
    cache: AccountCache,
}

impl AccountsDb {
    pub fn new(path: &str, cache_size: usize) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(1000);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self {
            db: Arc::new(db),
            cache: AccountCache::new(cache_size),
        })
    }

    /// Store an account
    pub fn store(&self, account: &OracleAccount) -> Result<()> {
        // Serialize account
        let data = bincode::serialize(account)?;
        
        // Write to RocksDB
        self.db.put(&account.pubkey, &data)?;
        
        // Update cache
        self.cache.insert(account.clone());
        
        Ok(())
    }

    /// Load an account
    pub fn load(&self, pubkey: &[u8; 32]) -> Result<Option<OracleAccount>> {
        // Check cache first
        if let Some(account) = self.cache.get(pubkey) {
            return Ok(Some(account));
        }
        
        // Load from RocksDB
        if let Some(data) = self.db.get(pubkey)? {
            let account: OracleAccount = bincode::deserialize(&data)?;
            
            // Update cache
            self.cache.insert(account.clone());
            
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    /// Delete an account
    pub fn delete(&self, pubkey: &[u8; 32]) -> Result<()> {
        self.db.delete(pubkey)?;
        self.cache.remove(pubkey);
        Ok(())
    }

    /// Batch store accounts
    pub fn store_batch(&self, accounts: &[OracleAccount]) -> Result<()> {
        let mut batch = WriteBatch::default();
        
        for account in accounts {
            let data = bincode::serialize(account)?;
            batch.put(&account.pubkey, &data);
            
            // Update cache
            self.cache.insert(account.clone());
        }
        
        self.db.write(batch)?;
        Ok(())
    }

    /// Get all accounts (for iteration)
    pub fn iter_accounts(&self) -> Result<Vec<OracleAccount>> {
        let mut accounts = Vec::new();
        
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (_key, value) = item?;
            let account: OracleAccount = bincode::deserialize(&value)?;
            accounts.push(account);
        }
        
        Ok(accounts)
    }

    /// Get accounts by owner
    pub fn get_accounts_by_owner(&self, owner: &[u8; 32]) -> Result<Vec<OracleAccount>> {
        let all_accounts = self.iter_accounts()?;
        Ok(all_accounts
            .into_iter()
            .filter(|acc| &acc.owner == owner)
            .collect())
    }

    /// Flush cache to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Compact database
    pub fn compact(&self) -> Result<()> {
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        Ok(())
    }

    /// Get database size
    pub fn size(&self) -> Result<u64> {
        // Approximate size
        let mut size = 0u64;
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (_key, value) = item?;
            size += value.len() as u64;
        }
        Ok(size)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache stats
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_account_cache() {
        let cache = AccountCache::new(10);
        
        let account = OracleAccount {
            pubkey: [1u8; 32],
            data: vec![1, 2, 3],
            lamports: 1000,
            owner: [0u8; 32],
            last_updated: 1000,
        };
        
        cache.insert(account.clone());
        assert_eq!(cache.len(), 1);
        
        let loaded = cache.get(&[1u8; 32]);
        assert!(loaded.is_some());
    }

    #[test]
    fn test_accounts_db() {
        let temp_dir = TempDir::new().unwrap();
        let db = AccountsDb::new(temp_dir.path().to_str().unwrap(), 100).unwrap();
        
        let account = OracleAccount {
            pubkey: [1u8; 32],
            data: vec![1, 2, 3],
            lamports: 1000,
            owner: [0u8; 32],
            last_updated: 1000,
        };
        
        db.store(&account).unwrap();
        
        let loaded = db.load(&[1u8; 32]).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().lamports, 1000);
    }

    #[test]
    fn test_batch_store() {
        let temp_dir = TempDir::new().unwrap();
        let db = AccountsDb::new(temp_dir.path().to_str().unwrap(), 100).unwrap();
        
        let accounts: Vec<OracleAccount> = (0..10)
            .map(|i| OracleAccount {
                pubkey: [i; 32],
                data: vec![i],
                lamports: i as u64 * 1000,
                owner: [0u8; 32],
                last_updated: 1000,
            })
            .collect();
        
        db.store_batch(&accounts).unwrap();
        
        for i in 0..10 {
            let loaded = db.load(&[i; 32]).unwrap();
            assert!(loaded.is_some());
        }
    }
}

