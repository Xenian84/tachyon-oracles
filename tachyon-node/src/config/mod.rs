use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use std::path::Path;
use std::fs;
use tracing::info;

use crate::crypto;

fn default_keypair() -> Keypair {
    Keypair::new()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node identity keypair
    #[serde(skip, default = "default_keypair")]
    pub identity: Keypair,
    
    /// Path to keypair file
    pub keypair_path: String,
    
    /// X1 RPC URL
    pub rpc_url: String,
    
    /// Tachyon program ID
    pub program_id: String,
    
    /// L2 State Compression program ID
    pub l2_program_id: String,
    
    /// Gossip network port
    pub gossip_port: u16,
    
    /// API server port
    pub api_port: u16,
    
    /// Price update interval (ms)
    pub update_interval_ms: u64,
    
    /// Batch interval (ms)
    pub batch_interval_ms: u64,
    
    /// Minimum publishers for quorum
    pub min_publishers: u8,
    
    /// Assets to track
    pub assets: Vec<AssetConfig>,
    
    /// Exchange API keys (optional)
    pub exchanges: ExchangeConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetConfig {
    pub symbol: String,
    pub exchanges: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub binance_api_key: Option<String>,
    pub coinbase_api_key: Option<String>,
    pub kraken_api_key: Option<String>,
}

impl NodeConfig {
    pub fn load(path: &str) -> Result<Self> {
        let expanded_path = shellexpand::tilde(path).to_string();
        let content = fs::read_to_string(&expanded_path)
            .with_context(|| format!("Failed to read config file: {}", expanded_path))?;
        
        let mut config: NodeConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        // Load keypair
        config.identity = crypto::load_keypair(&config.keypair_path)?;
        
        Ok(config)
    }
    
    pub fn save(&self, path: &str) -> Result<()> {
        let expanded_path = shellexpand::tilde(path).to_string();
        let content = toml::to_string_pretty(self)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(&expanded_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&expanded_path, content)
            .with_context(|| format!("Failed to write config file: {}", expanded_path))?;
        
        Ok(())
    }
}

pub async fn init_node(
    keypair_path: String,
    rpc_url: String,
    gossip_port: u16,
    api_port: u16,
) -> Result<()> {
    let expanded_keypair = shellexpand::tilde(&keypair_path).to_string();
    
    // Create config directory
    let config_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join(".config/tachyon");
    fs::create_dir_all(&config_dir)?;
    
    // Generate or load keypair
    let identity = if Path::new(&expanded_keypair).exists() {
        info!("📂 Loading existing keypair from {}", expanded_keypair);
        crypto::load_keypair(&expanded_keypair)?
    } else {
        info!("🔑 Generating new keypair...");
        let keypair = Keypair::new();
        crypto::save_keypair(&keypair, &expanded_keypair)?;
        info!("✅ Keypair saved to {}", expanded_keypair);
        keypair
    };
    
    let node_pubkey = identity.pubkey();
    info!("🔑 Node Identity: {}", node_pubkey);
    
    // Create default config
    let config = NodeConfig {
        identity,
        keypair_path: keypair_path.clone(),
        rpc_url,
        program_id: "TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1".to_string(),
        l2_program_id: "L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx".to_string(),
        gossip_port,
        api_port,
        update_interval_ms: 1000, // 1 second
        batch_interval_ms: 100,    // 100ms batches
        min_publishers: 3,
        assets: vec![
            AssetConfig { symbol: "BTC/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "ETH/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "SOL/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "AVAX/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "MATIC/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "BNB/USD".to_string(), exchanges: vec!["binance".to_string()] },
            AssetConfig { symbol: "XRP/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "ADA/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
            AssetConfig { symbol: "DOT/USD".to_string(), exchanges: vec!["binance".to_string(), "coinbase".to_string()] },
        ],
        exchanges: ExchangeConfig {
            binance_api_key: None,
            coinbase_api_key: None,
            kraken_api_key: None,
        },
    };
    
    // Save config
    let config_path = config_dir.join("node-config.toml");
    config.save(config_path.to_str().unwrap())?;
    
    info!("✅ Configuration saved to {}", config_path.display());
    info!("");
    info!("🚀 Next steps:");
    info!("  1. Fund your node wallet: {}", node_pubkey);
    info!("  2. Stake TACH tokens: tachyon-node stake --amount 1000");
    info!("  3. Start your node: tachyon-node start");
    
    Ok(())
}

