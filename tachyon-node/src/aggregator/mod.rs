#![allow(dead_code)]
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, debug};

use crate::config::NodeConfig;
use crate::fetcher::PriceUpdate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleBatch {
    pub root: String,
    pub timestamp: i64,
    pub feeds: Vec<FeedData>,
    pub tree: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedData {
    pub asset_id: String,
    pub price: i64,
    pub confidence: i64,
    pub timestamp: i64,
    pub publishers: Vec<String>,
}

pub async fn start_aggregator(
    config: Arc<NodeConfig>,
    mut price_rx: mpsc::Receiver<PriceUpdate>,
    mut gossip_rx: mpsc::Receiver<PriceUpdate>,
    batch_tx: mpsc::Sender<MerkleBatch>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    info!("🌳 Starting local aggregator...");
    
    let mut price_cache: HashMap<String, Vec<PriceUpdate>> = HashMap::new();
    let mut ticker = interval(Duration::from_millis(config.batch_interval_ms));
    
    loop {
        tokio::select! {
            // Receive local price updates
            Some(update) = price_rx.recv() => {
                price_cache.entry(update.asset.clone())
                    .or_insert_with(Vec::new)
                    .push(update);
            }
            
            // Receive gossip price updates from other nodes
            Some(update) = gossip_rx.recv() => {
                price_cache.entry(update.asset.clone())
                    .or_insert_with(Vec::new)
                    .push(update);
            }
            
            // Build Merkle batch every interval
            _ = ticker.tick() => {
                if price_cache.is_empty() {
                    continue;
                }
                
                let batch = build_merkle_batch(&price_cache, config.min_publishers);
                
                if !batch.feeds.is_empty() {
                    debug!("🌳 Built Merkle batch with {} feeds, root: {}",
                        batch.feeds.len(), &batch.root[..8]);
                    
                    if let Err(e) = batch_tx.send(batch).await {
                        tracing::error!("Failed to send batch: {}", e);
                    }
                }
                
                // Clear cache after batching
                price_cache.clear();
            }
            
            _ = shutdown.recv() => {
                info!("🌳 Aggregator shutting down...");
                break;
            }
        }
    }
    
    Ok(())
}

fn build_merkle_batch(
    price_cache: &HashMap<String, Vec<PriceUpdate>>,
    min_publishers: u8,
) -> MerkleBatch {
    let mut feeds = Vec::new();
    
    for (asset, updates) in price_cache {
        // Group by publisher
        let mut publisher_prices: HashMap<String, f64> = HashMap::new();
        
        for update in updates {
            publisher_prices.insert(update.node_pubkey.clone(), update.price);
        }
        
        // Check if we have enough publishers
        if publisher_prices.len() < min_publishers as usize {
            continue;
        }
        
        // Calculate median price
        let mut prices: Vec<f64> = publisher_prices.values().copied().collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if prices.len() % 2 == 0 {
            let mid = prices.len() / 2;
            (prices[mid - 1] + prices[mid]) / 2.0
        } else {
            prices[prices.len() / 2]
        };
        
        // Calculate confidence (inverse of std dev)
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();
        let confidence = if std_dev > 0.0 {
            1.0 / (1.0 + std_dev / mean)
        } else {
            1.0
        };
        
        // Convert to fixed-point integers (9 decimals)
        let price_i64 = (median * 1_000_000_000.0) as i64;
        let conf_i64 = (confidence * 1_000_000_000.0) as i64;
        
        feeds.push(FeedData {
            asset_id: asset.clone(),
            price: price_i64,
            confidence: conf_i64,
            timestamp: chrono::Utc::now().timestamp(),
            publishers: publisher_prices.keys().cloned().collect(),
        });
    }
    
    // Build Merkle tree
    let tree = build_merkle_tree(&feeds);
    let root = tree.last().unwrap_or(&String::new()).clone();
    
    MerkleBatch {
        root,
        timestamp: chrono::Utc::now().timestamp(),
        feeds,
        tree,
    }
}

fn build_merkle_tree(feeds: &[FeedData]) -> Vec<String> {
    if feeds.is_empty() {
        return vec![];
    }
    
    // Create leaf hashes
    let mut current_level: Vec<String> = feeds.iter()
        .map(|feed| {
            let data = format!("{}:{}:{}:{}",
                feed.asset_id, feed.price, feed.confidence, feed.timestamp);
            hash_data(data.as_bytes())
        })
        .collect();
    
    let mut tree = current_level.clone();
    
    // Build tree bottom-up
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                chunk[0].clone()
            };
            
            let parent_hash = hash_data(combined.as_bytes());
            next_level.push(parent_hash);
        }
        
        tree.extend(next_level.clone());
        current_level = next_level;
    }
    
    tree
}

fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn get_merkle_proof(tree: &[String], leaf_index: usize) -> Vec<String> {
    let mut proof = Vec::new();
    let mut index = leaf_index;
    let mut level_size = (tree.len() + 1) / 2;
    
    while level_size > 1 {
        let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        
        if sibling_index < level_size {
            proof.push(tree[sibling_index].clone());
        }
        
        index /= 2;
        level_size = (level_size + 1) / 2;
    }
    
    proof
}

