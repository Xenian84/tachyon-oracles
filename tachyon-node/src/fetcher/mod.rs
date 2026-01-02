use std::sync::Arc;
use solana_sdk::signer::Signer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

use crate::config::NodeConfig;

// Robust fetcher with outlier detection, circuit breaker, retry logic
pub mod robust_fetcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub asset: String,
    pub price: f64,
    pub confidence: f64,
    pub timestamp: i64,
    pub exchange: String,
    pub node_pubkey: String,
}

pub async fn start_price_fetcher(
    config: Arc<NodeConfig>,
    price_tx: mpsc::Sender<PriceUpdate>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    info!("📊 Starting price fetcher...");
    info!("📊 Configured assets: {:?}", config.assets.iter().map(|a| &a.symbol).collect::<Vec<_>>());
    info!("📊 Update interval: {}ms", config.update_interval_ms);
    
    let mut ticker = interval(Duration::from_millis(config.update_interval_ms));
    let node_pubkey = config.identity.pubkey().to_string();
    
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                info!("📊 Tick! Fetching prices for {} assets...", config.assets.len());
                // Fetch prices for all configured assets
                for asset in &config.assets {
                    info!("📊 Fetching {} from {:?}...", asset.symbol, asset.exchanges);
                    let prices = fetch_asset_prices(&asset.symbol, &asset.exchanges).await;
                    info!("📊 Got {} prices for {}: {:?}", prices.len(), asset.symbol, prices);
                    
                    if prices.is_empty() {
                        warn!("⚠️  No prices fetched for {}", asset.symbol);
                        continue;
                    }
                    
                    // Calculate median price and confidence
                    let (median, confidence) = calculate_median_and_confidence(&prices);
                    info!("📊 {} median price: ${:.2} (confidence: {:.2}%)", asset.symbol, median, confidence * 100.0);
                    
                    let update = PriceUpdate {
                        asset: asset.symbol.clone(),
                        price: median,
                        confidence,
                        timestamp: chrono::Utc::now().timestamp(),
                        exchange: "aggregated".to_string(),
                        node_pubkey: node_pubkey.clone(),
                    };
                    
                    if let Err(e) = price_tx.send(update).await {
                        error!("Failed to send price update: {}", e);
                    } else {
                        info!("✅ Sent price update for {}", asset.symbol);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("📊 Price fetcher shutting down...");
                break;
            }
        }
    }
    
    Ok(())
}

async fn fetch_asset_prices(symbol: &str, exchanges: &[String]) -> Vec<f64> {
    let mut prices = Vec::new();
    
    for exchange in exchanges {
        match fetch_from_exchange(symbol, exchange).await {
            Ok(price) => prices.push(price),
            Err(e) => warn!("Failed to fetch {} from {}: {}", symbol, exchange, e),
        }
    }
    
    prices
}

async fn fetch_from_exchange(symbol: &str, exchange: &str) -> Result<f64> {
    match exchange {
        "binance" => fetch_binance(symbol).await,
        "coinbase" => fetch_coinbase(symbol).await,
        "kraken" => fetch_kraken(symbol).await,
        "okx" => fetch_okx(symbol).await,
        "bybit" => fetch_bybit(symbol).await,
        _ => Err(anyhow::anyhow!("Unknown exchange: {}", exchange)),
    }
}

async fn fetch_binance(symbol: &str) -> Result<f64> {
    let binance_symbol = symbol.replace("/", "");
    let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", binance_symbol);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    #[derive(Deserialize)]
    struct BinanceResponse {
        price: Option<String>,
        code: Option<i32>,
        msg: Option<String>,
    }
    
    let data: BinanceResponse = response.json().await?;
    
    // Check for error response (geo-blocking, etc.)
    if let Some(code) = data.code {
        return Err(anyhow::anyhow!("Binance API error {}: {}", code, data.msg.unwrap_or_default()));
    }
    
    // Parse price
    let price_str = data.price.ok_or_else(|| anyhow::anyhow!("Missing price field"))?;
    let price: f64 = price_str.parse()?;
    
    Ok(price)
}

async fn fetch_coinbase(symbol: &str) -> Result<f64> {
    let coinbase_symbol = symbol.replace("/", "-");
    let url = format!("https://api.coinbase.com/v2/prices/{}/spot", coinbase_symbol);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    #[derive(Deserialize)]
    struct CoinbaseData {
        amount: String,
    }
    
    #[derive(Deserialize)]
    struct CoinbaseResponse {
        data: CoinbaseData,
    }
    
    let data: CoinbaseResponse = response.json().await?;
    let price: f64 = data.data.amount.parse()?;
    
    Ok(price)
}

async fn fetch_kraken(symbol: &str) -> Result<f64> {
    let kraken_symbol = symbol.replace("/", "");
    let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", kraken_symbol);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    #[derive(Deserialize)]
    struct KrakenResult {
        c: Vec<String>, // Last trade closed array [price, lot volume]
    }
    
    #[derive(Deserialize)]
    struct KrakenResponse {
        result: std::collections::HashMap<String, KrakenResult>,
    }
    
    let data: KrakenResponse = response.json().await?;
    
    if let Some((_, result)) = data.result.iter().next() {
        if let Some(price_str) = result.c.first() {
            let price: f64 = price_str.parse()?;
            return Ok(price);
        }
    }
    
    Err(anyhow::anyhow!("Failed to parse Kraken response"))
}

async fn fetch_okx(symbol: &str) -> Result<f64> {
    // OKX uses format like "BTC-USDT"
    let okx_symbol = symbol.replace("/", "-");
    let url = format!("https://www.okx.com/api/v5/market/ticker?instId={}", okx_symbol);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    #[derive(Deserialize)]
    struct OkxData {
        #[serde(rename = "last")]
        last: String,
    }
    
    #[derive(Deserialize)]
    struct OkxResponse {
        code: String,
        data: Vec<OkxData>,
    }
    
    let data: OkxResponse = response.json().await?;
    
    if data.code != "0" {
        return Err(anyhow::anyhow!("OKX API error: code {}", data.code));
    }
    
    if let Some(ticker) = data.data.first() {
        let price: f64 = ticker.last.parse()?;
        return Ok(price);
    }
    
    Err(anyhow::anyhow!("Failed to parse OKX response"))
}

async fn fetch_bybit(symbol: &str) -> Result<f64> {
    // Bybit uses format like "BTCUSDT"
    let bybit_symbol = symbol.replace("/", "");
    let url = format!("https://api.bybit.com/v5/market/tickers?category=spot&symbol={}", bybit_symbol);
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    #[derive(Deserialize)]
    struct BybitTicker {
        #[serde(rename = "lastPrice")]
        last_price: String,
    }
    
    #[derive(Deserialize)]
    struct BybitResult {
        list: Vec<BybitTicker>,
    }
    
    #[derive(Deserialize)]
    struct BybitResponse {
        #[serde(rename = "retCode")]
        ret_code: i32,
        result: BybitResult,
    }
    
    let data: BybitResponse = response.json().await?;
    
    if data.ret_code != 0 {
        return Err(anyhow::anyhow!("Bybit API error: code {}", data.ret_code));
    }
    
    if let Some(ticker) = data.result.list.first() {
        let price: f64 = ticker.last_price.parse()?;
        return Ok(price);
    }
    
    Err(anyhow::anyhow!("Failed to parse Bybit response"))
}

fn calculate_median_and_confidence(prices: &[f64]) -> (f64, f64) {
    if prices.is_empty() {
        return (0.0, 0.0);
    }
    
    let mut sorted = prices.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let median = if sorted.len() % 2 == 0 {
        let mid = sorted.len() / 2;
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    
    // Calculate confidence as inverse of standard deviation
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let variance = sorted.iter()
        .map(|p| (p - mean).powi(2))
        .sum::<f64>() / sorted.len() as f64;
    let std_dev = variance.sqrt();
    
    // Confidence is higher when std_dev is lower
    let confidence = if std_dev > 0.0 {
        1.0 / (1.0 + std_dev / mean)
    } else {
        1.0
    };
    
    (median, confidence)
}

