use sha2::{Digest, Sha256};

/// Hash an asset ID string to get the canonical 32-byte identifier
pub fn hash_asset_id(asset_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(asset_id.as_bytes());
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Calculate median of a sorted slice
pub fn median_i64(sorted: &[i64]) -> i64 {
    let len = sorted.len();
    if len == 0 {
        return 0;
    }
    if len % 2 == 0 {
        // Even number: average of two middle elements
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2
    } else {
        // Odd number: middle element
        sorted[len / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_odd() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(median_i64(&data), 3);
    }

    #[test]
    fn test_median_even() {
        let data = vec![1, 2, 3, 4];
        assert_eq!(median_i64(&data), 2); // (2+3)/2 = 2 (integer division)
    }

    #[test]
    fn test_hash_asset_id() {
        let hash1 = hash_asset_id("BTC/USD");
        let hash2 = hash_asset_id("BTC/USD");
        let hash3 = hash_asset_id("ETH/USD");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}

