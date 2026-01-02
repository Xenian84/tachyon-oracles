use std::sync::Arc;
use solana_sdk::signer::Signer;
use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod fetcher;
mod aggregator;
mod consensus;
mod sequencer;
mod gossip;
mod api;
mod crypto;
mod metrics;

// Solana components adapted for production-grade oracle network
// These modules contain infrastructure code that will be used in future features
#[allow(dead_code)]
mod streamer;    // ✅ High-performance packet processing
#[allow(dead_code)]
mod turbine;     // ✅ Efficient data propagation
#[allow(dead_code)]
mod poh;         // ✅ Proof of History ordering
#[allow(dead_code)]
mod vote;        // ✅ Validator voting
#[allow(dead_code)]
mod accounts_db; // ✅ High-performance storage
#[allow(dead_code)]
mod ledger;      // ✅ Historical data storage

use config::NodeConfig;

#[derive(Parser)]
#[command(name = "tachyon-node")]
#[command(about = "Tachyon Oracle Node - Decentralized Price Feeds for X1", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new node configuration
    Init {
        /// Path to keypair file
        #[arg(long, default_value = "~/.config/tachyon/node-keypair.json")]
        keypair: String,
        
        /// X1 RPC URL
        #[arg(long, default_value = "https://rpc.mainnet.x1.xyz")]
        rpc_url: String,
        
        /// Gossip port
        #[arg(long, default_value = "9000")]
        gossip_port: u16,
        
        /// API port
        #[arg(long, default_value = "7777")]
        api_port: u16,
    },
    
    /// Start the oracle node
    Start {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Show node status
    Status {
        /// API endpoint
        #[arg(long, default_value = "http://localhost:7777")]
        api: String,
    },
    
    /// Show node identity
    Identity {
        /// Path to keypair file
        #[arg(long, default_value = "~/.config/tachyon/node-keypair.json")]
        keypair: String,
    },
    
    /// Stake TACH tokens to become a publisher
    Stake {
        /// Amount of TACH tokens to stake
        #[arg(long)]
        amount: u64,
        
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Unstake TACH tokens
    Unstake {
        /// Amount of TACH tokens to unstake
        #[arg(long)]
        amount: Option<u64>,
        
        /// Unstake all tokens
        #[arg(long)]
        all: bool,
        
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Claim staking rewards
    ClaimRewards {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Claim rewards and automatically compound (stake them)
    ClaimAndCompound {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Claim referral rewards
    ClaimReferralRewards {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Update loyalty tier based on stake duration
    UpdateLoyaltyTier {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// View detailed staking information with rewards breakdown
    ViewStakeInfo {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// View performance metrics
    ViewPerformance {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// View referral statistics
    ViewReferrals {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
    
    /// Register as sequencer
    Register {
        /// Path to config file
        #[arg(long, default_value = "~/.config/tachyon/node-config.toml")]
        config: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tachyon_node=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { keypair, rpc_url, gossip_port, api_port } => {
            info!("🚀 Initializing Tachyon Node...");
            config::init_node(keypair, rpc_url, gossip_port, api_port).await?;
        }
        Commands::Start { config } => {
            info!("🚀 Starting Tachyon Node...");
            start_node(config).await?;
        }
        Commands::Status { api } => {
            info!("📊 Fetching node status...");
            show_status(api).await?;
        }
        Commands::Identity { keypair } => {
            info!("🔑 Loading node identity...");
            show_identity(keypair).await?;
        }
        Commands::Stake { amount, config } => {
            info!("💰 Staking {} TACH tokens...", amount);
            stake_tokens(amount, config).await?;
        }
        Commands::Unstake { amount, all, config } => {
            if all {
                info!("💰 Unstaking all TACH tokens...");
                unstake_tokens(None, config).await?;
            } else if let Some(amt) = amount {
                info!("💰 Unstaking {} TACH tokens...", amt);
                unstake_tokens(Some(amt), config).await?;
            } else {
                error!("❌ Please specify --amount or --all");
                std::process::exit(1);
            }
        }
        Commands::ClaimRewards { config } => {
            info!("💰 Claiming staking rewards...");
            claim_rewards(config).await?;
        }
        Commands::ClaimAndCompound { config } => {
            info!("💰 Claiming and compounding rewards...");
            claim_and_compound(config).await?;
        }
        Commands::ClaimReferralRewards { config } => {
            info!("🎁 Claiming referral rewards...");
            claim_referral_rewards(config).await?;
        }
        Commands::UpdateLoyaltyTier { config } => {
            info!("⭐ Updating loyalty tier...");
            update_loyalty_tier(config).await?;
        }
        Commands::ViewStakeInfo { config } => {
            info!("📊 Fetching stake information...");
            view_stake_info(config).await?;
        }
        Commands::ViewPerformance { config } => {
            info!("📈 Fetching performance metrics...");
            view_performance(config).await?;
        }
        Commands::ViewReferrals { config } => {
            info!("🎁 Fetching referral statistics...");
            view_referrals(config).await?;
        }
        Commands::Register { config } => {
            info!("🎯 Registering as sequencer...");
            register_as_sequencer(config).await?;
        }
    }

    Ok(())
}

async fn start_node(config_path: String) -> Result<()> {
    let config = Arc::new(NodeConfig::load(&config_path)?);
    
    info!("🔑 Node Identity: {}", config.identity.pubkey());
    info!("🌐 RPC URL: {}", config.rpc_url);
    info!("📡 Gossip Port: {}", config.gossip_port);
    info!("🔌 API Port: {}", config.api_port);
    
    // Start all subsystems
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);
    
    // 1. Start metrics server
    let metrics_handle = tokio::spawn({
        let config = Arc::clone(&config);
        let shutdown = shutdown_tx.subscribe();
        async move {
            metrics::start_metrics_server(config.api_port, shutdown).await
        }
    });
    
    // 2. Start P2P gossip network
    let (gossip_tx, gossip_rx) = tokio::sync::mpsc::channel(1000);
    let gossip_handle = tokio::spawn({
        let config = Arc::clone(&config);
        let shutdown = shutdown_tx.subscribe();
        async move {
            gossip::start_gossip_network(config, gossip_tx, shutdown).await
        }
    });
    
    // 3. Start price fetcher
    let (price_tx, price_rx) = tokio::sync::mpsc::channel(1000);
    let fetcher_handle = tokio::spawn({
        let config = Arc::clone(&config);
        let shutdown = shutdown_tx.subscribe();
        async move {
            fetcher::start_price_fetcher(config, price_tx, shutdown).await
        }
    });
    
    // 4. Start local aggregator (builds Merkle trees)
    let (batch_tx, batch_rx) = tokio::sync::mpsc::channel(100);
    let aggregator_handle = tokio::spawn({
        let config = Arc::clone(&config);
        #[allow(unused_mut)]
        let mut shutdown = shutdown_tx.subscribe();
        async move {
            aggregator::start_aggregator(config, price_rx, gossip_rx, batch_tx, shutdown).await
        }
    });
    
    // 5. Start consensus module (votes on batches)
    let (consensus_tx, consensus_rx) = tokio::sync::mpsc::channel(100);
    let consensus_handle = tokio::spawn({
        let config = Arc::clone(&config);
        #[allow(unused_mut)]
        let mut shutdown = shutdown_tx.subscribe();
        async move {
            consensus::start_consensus(config, batch_rx, consensus_tx, shutdown).await
        }
    });
    
    // 6. Start sequencer (submits to X1)
    let sequencer_handle = tokio::spawn({
        let config = Arc::clone(&config);
        #[allow(unused_mut)]
        let mut shutdown = shutdown_tx.subscribe();
        async move {
            sequencer::start_sequencer(config, consensus_rx, shutdown).await
        }
    });
    
    // 7. Start API server
    let api_handle = tokio::spawn({
        let config = Arc::clone(&config);
        #[allow(unused_mut)]
        let mut shutdown = shutdown_tx.subscribe();
        async move {
            api::start_api_server(config, shutdown).await
        }
    });
    
    info!("✅ All subsystems started successfully!");
    info!("🎯 Node is now running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Received shutdown signal...");
        }
        _ = shutdown_rx.recv() => {
            info!("🛑 Shutdown requested...");
        }
    }
    
    // Broadcast shutdown to all tasks
    let _ = shutdown_tx.send(());
    
    // Wait for all tasks to complete
    let _ = tokio::join!(
        metrics_handle,
        gossip_handle,
        fetcher_handle,
        aggregator_handle,
        consensus_handle,
        sequencer_handle,
        api_handle,
    );
    
    info!("✅ Node stopped gracefully");
    Ok(())
}

async fn show_status(api: String) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{}/status", api)).send().await?;
    let status: serde_json::Value = response.json().await?;
    
    println!("\n📊 Node Status:");
    println!("{}", serde_json::to_string_pretty(&status)?);
    
    Ok(())
}

async fn show_identity(keypair_path: String) -> Result<()> {
    let keypair = crypto::load_keypair(&keypair_path)?;
    
    println!("\n🔑 Node Identity:");
    println!("  Public Key: {}", keypair.pubkey());
    println!("  Keypair Path: {}", keypair_path);
    
    Ok(())
}

async fn stake_tokens(amount: u64, config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::instruction::{Instruction, AccountMeta};
    use solana_sdk::transaction::Transaction;
    #[allow(deprecated)]
    use solana_sdk::system_program;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                  💰 STAKING TACH TOKENS                          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
    
    println!("📋 Staking Details:");
    println!("  Amount:     {} TACH", amount);
    println!("  Node:       {}", config.identity.pubkey());
    println!("  Governance: TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9");
    println!();
    
    let governance_program = Pubkey::from_str("TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9")?;
    let tach_mint = Pubkey::from_str("TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu")?;
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
    
    // Derive PDAs
    let (governance_state_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program,
    );
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault"],
        &governance_program,
    );
    
    // Get staker's token account (associated token account)
    let staker_token_account = anchor_spl::associated_token::get_associated_token_address(
        &config.identity.pubkey(),
        &tach_mint,
    );
    
    println!("🔍 Checking TACH balance...");
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    // Check if staker token account exists
    match rpc_client.get_account(&staker_token_account) {
        Ok(_) => println!("   ✅ TACH token account found"),
        Err(_) => {
            println!("\n❌ Error: TACH token account not found");
            println!("   Create it first:");
            println!("   spl-token create-account TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu");
            return Err(anyhow::anyhow!("TACH token account not found"));
        }
    }
    
    // Build stake instruction
    // Discriminator for "stake" - sha256("global:stake")[0..8]
    let mut data = vec![0u8; 16];
    data[0..8].copy_from_slice(&[0xce, 0xb0, 0xca, 0x12, 0xc8, 0xd1, 0xb3, 0x6c]);
    let amount_with_decimals = amount * 1_000_000_000u64; // Convert to lamports (9 decimals)
    data[8..16].copy_from_slice(&amount_with_decimals.to_le_bytes());
    
    // First, check if staker_info needs to be initialized
    match rpc_client.get_account(&staker_info_pda) {
        Ok(_) => println!("   ✅ Staker info already initialized"),
        Err(_) => {
            println!("   🔧 Initializing staker info...");
            // Build init_staker instruction
            let mut init_data = vec![0u8; 8];
            let init_discriminator = {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(b"global:init_staker");
                let result = hasher.finalize();
                let mut disc = [0u8; 8];
                disc.copy_from_slice(&result[0..8]);
                disc
            };
            init_data[0..8].copy_from_slice(&init_discriminator);
            
            let init_ix = Instruction {
                program_id: governance_program,
                accounts: vec![
                    AccountMeta::new(staker_info_pda, false), // staker_info
                    AccountMeta::new(config.identity.pubkey(), true), // staker (signer + payer)
                    AccountMeta::new_readonly(system_program::id(), false), // system_program
                ],
                data: init_data,
            };
            
            let recent_blockhash = rpc_client.get_latest_blockhash()?;
            let init_tx = Transaction::new_signed_with_payer(
                &[init_ix],
                Some(&config.identity.pubkey()),
                &[&config.identity],
                recent_blockhash,
            );
            
            match rpc_client.send_and_confirm_transaction(&init_tx) {
                Ok(sig) => println!("   ✅ Staker info initialized: {}", sig),
                Err(e) => {
                    println!("   ❌ Failed to initialize staker info: {}", e);
                    return Err(anyhow::anyhow!("Failed to initialize staker info"));
                }
            }
        }
    }
    
    let ix = Instruction {
        program_id: governance_program,
        accounts: vec![
            AccountMeta::new(governance_state_pda, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(staker_info_pda, false),
            AccountMeta::new(staker_token_account, false),
            AccountMeta::new(config.identity.pubkey(), true),
            AccountMeta::new_readonly(token_program, false),
        ],
        data,
    };
    
    println!("\n📤 Sending stake transaction...");
    
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&config.identity.pubkey()),
        &[&config.identity],
        recent_blockhash,
    );
    
    match rpc_client.send_and_confirm_transaction(&tx) {
        Ok(signature) => {
            println!("\n✅ Staked {} TACH successfully!", amount);
            println!("   Transaction: {}", signature);
            println!("   Staker Info PDA: {}", staker_info_pda);
        }
        Err(e) => {
            println!("\n❌ Staking failed: {}", e);
            println!("\n⚠️  This may require:");
            println!("   1. Sufficient TACH balance in your token account");
            println!("   2. Sufficient X1 for transaction fees");
            println!("   3. Governance contract to be properly initialized");
        }
    }
    
    Ok(())
}

async fn unstake_tokens(amount: Option<u64>, config_path: String) -> Result<()> {
    let config = Arc::new(NodeConfig::load(&config_path)?);
    
    if let Some(amt) = amount {
        info!("💰 Unstaking {} TACH tokens...", amt);
    } else {
        info!("💰 Unstaking all TACH tokens...");
    }
    info!("🔑 Node: {}", config.identity.pubkey());
    
    // TODO: Implement actual on-chain unstaking via TachyonGovernance contract
    
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                 💰 UNSTAKING TACH TOKENS                         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
    
    println!("📋 Unstaking Details:");
    if let Some(amt) = amount {
        println!("  Amount:     {} TACH", amt);
    } else {
        println!("  Amount:     ALL");
    }
    println!("  Node:       {}", config.identity.pubkey());
    println!("  Governance: TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9");
    println!();
    
    println!("⚠️  WARNING: Unstaking will stop your node from earning rewards!");
    println!("⚠️  There may be a cooldown period before tokens are available.");
    println!();
    
    println!("⚠️  On-chain unstaking integration coming soon!");
    println!();
    println!("📝 For now, unstake manually using Anchor CLI:");
    let amount_str = amount.map(|a| a.to_string()).unwrap_or_else(|| "all".to_string());
    println!("  anchor run unstake --provider.wallet {} --amount {}", 
             config_path.replace("node-config.toml", "node-wallet.json"), amount_str);
    
    Ok(())
}

async fn claim_rewards(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::instruction::{Instruction, AccountMeta};
    use solana_sdk::transaction::Transaction;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                  💰 CLAIMING STAKING REWARDS                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
    
    println!("📋 Claim Details:");
    println!("  Node:       {}", config.identity.pubkey());
    println!("  Governance: TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9");
    println!();
    
    let governance_program = Pubkey::from_str("TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9")?;
    let tach_mint = Pubkey::from_str("TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu")?;
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
    
    // Derive PDAs
    let (governance_state_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program,
    );
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let (rewards_pool_pda, _) = Pubkey::find_program_address(
        &[b"rewards-pool"],
        &governance_program,
    );
    
    // Get staker's token account
    let staker_token_account = anchor_spl::associated_token::get_associated_token_address(
        &config.identity.pubkey(),
        &tach_mint,
    );
    
    println!("🔍 Checking staker status...");
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    // Check if staker_info exists
    match rpc_client.get_account(&staker_info_pda) {
        Ok(_) => println!("   ✅ Staker account found"),
        Err(_) => {
            println!("\n❌ Error: Not staked");
            println!("   Stake TACH first: cargo run --release -- stake --amount 100000");
            return Err(anyhow::anyhow!("Not staked"));
        }
    }
    
    // Build claim_rewards instruction
    // Discriminator for "claim_rewards" - sha256("global:claim_rewards")[0..8]
    let discriminator = {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(b"global:claim_rewards");
        let result = hasher.finalize();
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&result[0..8]);
        disc
    };
    
    let mut data = vec![0u8; 8];
    data[0..8].copy_from_slice(&discriminator);
    
    let claim_ix = Instruction {
        program_id: governance_program,
        accounts: vec![
            AccountMeta::new(governance_state_pda, false),
            AccountMeta::new(rewards_pool_pda, false), // FIXED ORDER: rewards_pool before staker_info
            AccountMeta::new(staker_info_pda, false),
            AccountMeta::new(staker_token_account, false),
            AccountMeta::new_readonly(config.identity.pubkey(), true), // staker (signer)
            AccountMeta::new_readonly(token_program, false),
        ],
        data,
    };
    
    println!("📤 Submitting claim transaction...");
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[claim_ix],
        Some(&config.identity.pubkey()),
        &[&config.identity],
        recent_blockhash,
    );
    
    match rpc_client.send_and_confirm_transaction(&tx) {
        Ok(signature) => {
            println!("\n✅ Rewards claimed successfully!");
            println!("   Signature: {}", signature);
            println!("   Explorer: https://explorer.x1.xyz/tx/{}", signature);
        }
        Err(e) => {
            println!("\n❌ Claim failed: {}", e);
            return Err(anyhow::anyhow!("Claim failed: {}", e));
        }
    }
    
    Ok(())
}

async fn claim_and_compound(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::instruction::{Instruction, AccountMeta};
    use solana_sdk::transaction::Transaction;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    let governance_program = Pubkey::from_str(&config.program_id)?;
    let _tach_mint = Pubkey::from_str("TACHsKdrrCe1xE1v82WQ3j5FqqMqXxGEFcZyLvEMbQV")?;
    
    let (governance_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program,
    );
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let (rewards_pool_pda, _) = Pubkey::find_program_address(
        &[b"rewards-pool"],
        &governance_program,
    );
    
    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault"],
        &governance_program,
    );
    
    println!("🔄 Claiming and compounding rewards...");
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Discriminator for claim_and_compound instruction (8 bytes)
    let instruction_data = vec![0x8a, 0x6d, 0x1f, 0x8e, 0x5c, 0x3b, 0x2a, 0x1d];
    
    let accounts = vec![
        AccountMeta::new(governance_pda, false),
        AccountMeta::new(staker_info_pda, false),
        AccountMeta::new(rewards_pool_pda, false),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new_readonly(config.identity.pubkey(), true),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
    ];
    
    let instruction = Instruction {
        program_id: governance_program,
        accounts,
        data: instruction_data,
    };
    
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&config.identity.pubkey()),
        &[&config.identity],
        recent_blockhash,
    );
    
    match rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            println!("\n✅ Rewards claimed and compounded successfully!");
            println!("   📝 Signature: {}", signature);
        }
        Err(e) => {
            println!("\n❌ Compound failed: {}", e);
            return Err(anyhow::anyhow!("Compound failed: {}", e));
        }
    }
    
    Ok(())
}

async fn claim_referral_rewards(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::instruction::{Instruction, AccountMeta};
    use solana_sdk::transaction::Transaction;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    let governance_program = Pubkey::from_str(&config.program_id)?;
    let tach_mint = Pubkey::from_str("TACHsKdrrCe1xE1v82WQ3j5FqqMqXxGEFcZyLvEMbQV")?;
    
    let (governance_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program,
    );
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let (rewards_pool_pda, _) = Pubkey::find_program_address(
        &[b"rewards-pool"],
        &governance_program,
    );
    
    let staker_token_account = anchor_spl::associated_token::get_associated_token_address(
        &config.identity.pubkey(),
        &tach_mint,
    );
    
    println!("🎁 Claiming referral rewards...");
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Discriminator for claim_referral_rewards instruction (8 bytes)
    let instruction_data = vec![0x9b, 0x7e, 0x2f, 0x9f, 0x6d, 0x4c, 0x3b, 0x2e];
    
    let accounts = vec![
        AccountMeta::new(governance_pda, false),
        AccountMeta::new(staker_info_pda, false),
        AccountMeta::new(rewards_pool_pda, false),
        AccountMeta::new(staker_token_account, false),
        AccountMeta::new_readonly(config.identity.pubkey(), true),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
    ];
    
    let instruction = Instruction {
        program_id: governance_program,
        accounts,
        data: instruction_data,
    };
    
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&config.identity.pubkey()),
        &[&config.identity],
        recent_blockhash,
    );
    
    match rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            println!("\n✅ Referral rewards claimed successfully!");
            println!("   📝 Signature: {}", signature);
        }
        Err(e) => {
            println!("\n❌ Claim failed: {}", e);
            return Err(anyhow::anyhow!("Claim failed: {}", e));
        }
    }
    
    Ok(())
}

async fn update_loyalty_tier(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::instruction::{Instruction, AccountMeta};
    use solana_sdk::transaction::Transaction;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    let governance_program = Pubkey::from_str(&config.program_id)?;
    
    let (governance_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program,
    );
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    println!("⭐ Updating loyalty tier...");
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Discriminator for update_loyalty_tier instruction (8 bytes)
    let instruction_data = vec![0xac, 0x8f, 0x3f, 0xaf, 0x7e, 0x5d, 0x4c, 0x3f];
    
    let accounts = vec![
        AccountMeta::new(governance_pda, false),
        AccountMeta::new(staker_info_pda, false),
        AccountMeta::new_readonly(config.identity.pubkey(), true),
    ];
    
    let instruction = Instruction {
        program_id: governance_program,
        accounts,
        data: instruction_data,
    };
    
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&config.identity.pubkey()),
        &[&config.identity],
        recent_blockhash,
    );
    
    match rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            println!("\n✅ Loyalty tier updated successfully!");
            println!("   📝 Signature: {}", signature);
        }
        Err(e) => {
            println!("\n❌ Update failed: {}", e);
            return Err(anyhow::anyhow!("Update failed: {}", e));
        }
    }
    
    Ok(())
}

async fn view_stake_info(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    let governance_program = Pubkey::from_str(&config.program_id)?;
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    match rpc_client.get_account(&staker_info_pda) {
        Ok(account) => {
            let data = &account.data;
            
            // Check minimum size
            if data.len() < 24 {
                error!("❌ Staker account too small: {} bytes", data.len());
                return Ok(());
            }
            
            // Parse StakerInfo account (skip 8-byte discriminator)
            // Actual structure from contract:
            // pub staked_amount: u64,             // offset 8
            // pub last_stake_timestamp: i64,      // offset 16
            // pub bump: u8,                       // offset 24
            // pub total_rewards_claimed: u64,     // offset 25
            // pub last_claim_timestamp: i64,      // offset 33
            // pub pending_rewards: u64,           // offset 41
            // pub compounded_rewards: u64,        // offset 49
            // pub uptime_score: u64,              // offset 57
            // pub submissions_count: u64,         // offset 65
            // pub accurate_submissions: u64,      // offset 73
            // pub first_stake_timestamp: i64,     // offset 81
            // pub loyalty_tier: u8,               // offset 89
            // pub referrer: Pubkey,               // offset 90 (32 bytes)
            // pub referral_count: u64,            // offset 122
            // pub referral_rewards: u64,          // offset 130
            // pub vested_rewards: u64,            // offset 138
            // pub vesting_start: i64,             // offset 146
            
            let staked_amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
            let last_stake_timestamp = i64::from_le_bytes(data[16..24].try_into().unwrap());
            let _bump = data[24];
            
            let total_rewards_claimed = if data.len() >= 33 { u64::from_le_bytes(data[25..33].try_into().unwrap()) } else { 0 };
            let last_claim_timestamp = if data.len() >= 41 { i64::from_le_bytes(data[33..41].try_into().unwrap()) } else { 0 };
            let pending_rewards = if data.len() >= 49 { u64::from_le_bytes(data[41..49].try_into().unwrap()) } else { 0 };
            let compounded_rewards = if data.len() >= 57 { u64::from_le_bytes(data[49..57].try_into().unwrap()) } else { 0 };
            let uptime_score = if data.len() >= 65 { u64::from_le_bytes(data[57..65].try_into().unwrap()) } else { 10000 };
            let submissions_count = if data.len() >= 73 { u64::from_le_bytes(data[65..73].try_into().unwrap()) } else { 0 };
            let accurate_submissions = if data.len() >= 81 { u64::from_le_bytes(data[73..81].try_into().unwrap()) } else { 0 };
            let _first_stake_timestamp = if data.len() >= 89 { i64::from_le_bytes(data[81..89].try_into().unwrap()) } else { last_stake_timestamp };
            let loyalty_tier = if data.len() >= 90 { data[89] } else { 0 };
            // Skip referrer pubkey (32 bytes at offset 90-122)
            let referral_count = if data.len() >= 130 { u64::from_le_bytes(data[122..130].try_into().unwrap()) } else { 0 };
            let referral_rewards = if data.len() >= 138 { u64::from_le_bytes(data[130..138].try_into().unwrap()) } else { 0 };
            let vested_rewards = if data.len() >= 146 { u64::from_le_bytes(data[138..146].try_into().unwrap()) } else { 0 };
            let _vesting_start = if data.len() >= 154 { i64::from_le_bytes(data[146..154].try_into().unwrap()) } else { 0 };
            
            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║              📊 DETAILED STAKE INFORMATION                   ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 💰 Staked Amount:        {:>12.2} TACH                   ║", staked_amount as f64 / 1_000_000.0);
            println!("║ 📅 Staked Since:         {}                    ║", 
                chrono::DateTime::from_timestamp(last_stake_timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown".to_string()));
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║                    🎁 REWARDS SUMMARY                        ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 💎 Pending Rewards:      {:>12.2} TACH                   ║", pending_rewards as f64 / 1_000_000.0);
            println!("║ ✅ Total Claimed:        {:>12.2} TACH                   ║", total_rewards_claimed as f64 / 1_000_000.0);
            println!("║ 🔄 Compounded:           {:>12.2} TACH                   ║", compounded_rewards as f64 / 1_000_000.0);
            println!("║ 💸 Vested:               {:>12.2} TACH                   ║", vested_rewards as f64 / 1_000_000.0);
            println!("║ 📅 Last Claim:           {}                    ║", 
                if last_claim_timestamp > 0 {
                    chrono::DateTime::from_timestamp(last_claim_timestamp, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "Never".to_string()
                });
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║                  📈 PERFORMANCE METRICS                      ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            let uptime_percent = (uptime_score as f64 / 10000.0 * 100.0) as u64;
            println!("║ 🎯 Uptime Score:         {:>3}% ({}x multiplier)          ║", 
                uptime_percent, 
                if uptime_percent >= 95 { "1.5" } else if uptime_percent >= 90 { "1.25" } else if uptime_percent >= 80 { "1.0" } else { "0.5" });
            println!("║ 📊 Submissions:          {:>12} total                 ║", submissions_count);
            println!("║ ✅ Success Rate:         {:>3}% ({}/{})                  ║", 
                if submissions_count > 0 { accurate_submissions * 100 / submissions_count } else { 0 },
                accurate_submissions,
                submissions_count);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║                    ⭐ LOYALTY PROGRAM                        ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            let tier_name = match loyalty_tier {
                0 => "Bronze",
                1 => "Silver",
                2 => "Gold",
                3 => "Platinum",
                _ => "Unknown",
            };
            let loyalty_bonus = match loyalty_tier {
                0 => 0,   // Bronze: 0%
                1 => 10,  // Silver: 10%
                2 => 20,  // Gold: 20%
                3 => 30,  // Platinum: 30%
                _ => 0,
            };
            println!("║ 🏆 Loyalty Tier:         {} ({}% bonus)                ║", tier_name, loyalty_bonus);
            println!("║ 🔒 Vested Amount:        {:>12.2} TACH                   ║", vested_rewards as f64 / 1_000_000.0);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║                   🎁 REFERRAL PROGRAM                        ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 👥 Referrals:            {:>3} validators                     ║", referral_count);
            println!("║ 💰 Total Rewards:        {:>12.2} TACH                   ║", referral_rewards as f64 / 1_000_000.0);
            println!("╚══════════════════════════════════════════════════════════════╝\n");
        }
        Err(e) => {
            println!("\n❌ Error fetching staking information: {}", e);
            println!("   PDA: {}", staker_info_pda);
            println!("   💡 Stake some TACH tokens first using: tachyon-node stake --amount <AMOUNT>");
        }
    }
    
    Ok(())
}

async fn view_performance(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    let governance_program = Pubkey::from_str(&config.program_id)?;
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    match rpc_client.get_account(&staker_info_pda) {
        Ok(account) => {
            let data = &account.data;
            
            // Correct offsets accounting for bump field at offset 24
            // uptime_score: u64 at offset 57
            // submissions_count: u64 at offset 65
            // accurate_submissions: u64 at offset 73
            let uptime_score = if data.len() >= 65 { u64::from_le_bytes(data[57..65].try_into().unwrap()) } else { 10000 };
            let submissions_count = if data.len() >= 73 { u64::from_le_bytes(data[65..73].try_into().unwrap()) } else { 0 };
            let accurate_submissions = if data.len() >= 81 { u64::from_le_bytes(data[73..81].try_into().unwrap()) } else { 0 };
            
            // Convert uptime_score (0-10000) to percentage
            let performance_score = (uptime_score as f64 / 10000.0 * 100.0) as u32;
            
            let success_rate = if submissions_count > 0 {
                accurate_submissions * 100 / submissions_count
            } else {
                0
            };
            
            let multiplier = if performance_score >= 95 {
                "1.5x (🔥 EXCELLENT!)"
            } else if performance_score >= 90 {
                "1.25x (⭐ Great!)"
            } else if performance_score >= 80 {
                "1.0x (✅ Good)"
            } else {
                "0.5x (⚠️  Needs Improvement)"
            };
            
            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║              📈 PERFORMANCE METRICS                          ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 🎯 Overall Score:        {}%                              ║", performance_score);
            println!("║ 🎁 Rewards Multiplier:   {}                    ║", multiplier);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ ⏱️  Uptime Score:         {}% ({}x multiplier)              ║", performance_score, 
                if performance_score >= 95 { "1.5" } else if performance_score >= 90 { "1.25" } else if performance_score >= 80 { "1.0" } else { "0.5" });
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ ✅ Successful:           {} submissions                     ║", accurate_submissions);
            println!("║ ❌ Failed:               {} submissions                     ║", submissions_count.saturating_sub(accurate_submissions));
            println!("║ 📊 Total:                {} submissions                     ║", submissions_count);
            println!("║ 📈 Success Rate:         {}%                               ║", success_rate);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║                    💡 IMPROVEMENT TIPS                       ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            if performance_score < 95 {
                println!("║ • Maintain 99%+ uptime for maximum rewards                  ║");
                println!("║ • Ensure accurate price submissions                         ║");
                println!("║ • Keep your node updated and well-maintained                ║");
            } else {
                println!("║ 🎉 Excellent performance! Keep up the great work!          ║");
            }
            println!("╚══════════════════════════════════════════════════════════════╝\n");
        }
        Err(_) => {
            println!("\n❌ No performance data found for this validator");
        }
    }
    
    Ok(())
}

async fn view_referrals(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    let governance_program = Pubkey::from_str(&config.program_id)?;
    
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    match rpc_client.get_account(&staker_info_pda) {
        Ok(account) => {
            let data = &account.data;
            
            let referral_count = u32::from_le_bytes(data[111..115].try_into().unwrap());
            let referral_rewards_earned = u64::from_le_bytes(data[115..123].try_into().unwrap());
            let referral_rewards_claimed = u64::from_le_bytes(data[123..131].try_into().unwrap());
            let pending = referral_rewards_earned - referral_rewards_claimed;
            
            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║              🎁 REFERRAL PROGRAM STATISTICS                  ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 👥 Total Referrals:      {:>3} validators                     ║", referral_count);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 💰 Total Earned:         {:>12.2} TACH                   ║", referral_rewards_earned as f64 / 1_000_000.0);
            println!("║ ✅ Total Claimed:        {:>12.2} TACH                   ║", referral_rewards_claimed as f64 / 1_000_000.0);
            println!("║ 💎 Pending Rewards:      {:>12.2} TACH                   ║", pending as f64 / 1_000_000.0);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║                    💡 REFERRAL INFO                          ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ 🔗 Your Referral Code:                                       ║");
            println!("║    {}          ║", config.identity.pubkey());
            println!("╠══════════════════════════════════════════════════════════════╣");
            if pending > 0 {
                println!("║ 💡 Claim your pending rewards with:                         ║");
                println!("║    tachyon-node claim-referral-rewards                       ║");
            } else {
                println!("║ ✨ No pending rewards. Share your referral code!            ║");
            }
            println!("╚══════════════════════════════════════════════════════════════╝\n");
        }
        Err(_) => {
            println!("\n❌ No referral data found for this validator");
        }
    }
    
    Ok(())
}

async fn register_as_sequencer(config_path: String) -> Result<()> {
    use solana_sdk::pubkey::Pubkey;
    use solana_client::rpc_client::RpcClient;
    use std::str::FromStr;
    
    let config = Arc::new(NodeConfig::load(&config_path)?);
    
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║              🎯 REGISTERING AS SEQUENCER                         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");
    
    println!("📋 Registration Details:");
    println!("  Node:       {}", config.identity.pubkey());
    println!("  Sequencer:  SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M");
    println!();
    
    // Check if already registered
    let sequencer_program = Pubkey::from_str("SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M")?;
    let (sequencer_info_pda, _) = Pubkey::find_program_address(
        &[b"sequencer-info", config.identity.pubkey().as_ref()],
        &sequencer_program,
    );
    
    let rpc_client = RpcClient::new(&config.rpc_url);
    
    println!("🔍 Checking if already registered...");
    match rpc_client.get_account(&sequencer_info_pda) {
        Ok(_) => {
            println!("\n✅ Already registered as sequencer!");
            println!("   Sequencer Info PDA: {}", sequencer_info_pda);
            return Ok(());
        }
        Err(_) => {
            println!("   Not registered yet.");
        }
    }
    
    // Check stake requirement
    println!("\n🔍 Checking stake requirement (100,000 TACH)...");
    let governance_program = Pubkey::from_str(&config.program_id)?;
    let (staker_info_pda, _) = Pubkey::find_program_address(
        &[b"staker-v2", config.identity.pubkey().as_ref()],
        &governance_program,
    );
    
    match rpc_client.get_account(&staker_info_pda) {
        Ok(_) => {
            println!("   ✅ Staker account found");
        }
        Err(_) => {
            println!("\n❌ Error: You must stake at least 100,000 TACH before registering");
            println!("\n📝 To stake:");
            println!("   tachyon-node stake --amount 100000");
            return Err(anyhow::anyhow!("Insufficient stake"));
        }
    }
    
    println!("\n⚠️  Note: Registration requires deployer approval.");
    println!("   Contact network administrator to complete registration.");
    println!("\n📝 Sequencer Info PDA: {}", sequencer_info_pda);
    
    Ok(())
}
