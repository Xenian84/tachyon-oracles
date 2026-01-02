// Robust Price Fetcher - Production-grade price aggregation
// Includes: Outlier detection, circuit breaker, retry logic, weighted averaging
// Infrastructure code for future use
#![allow(dead_code)]

use std::collections::HashMap;
use std::time::Duration;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{warn, info};

/// Circuit breaker state
#[derive(Clone, Debug, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Stop fetching (too many failures)
    HalfOpen, // Try again (testing if recovered)
}

/// Circuit breaker for exchange API calls
#[derive(Clone, Debug)]
pub struct CircuitBreaker {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub threshold: u32,
    pub timeout_secs: u64,
    pub last_failure_time: Option<i64>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            threshold,
            timeout_secs,
            last_failure_time: None,
        }
    }

    /// Record a successful call
    pub fn record_success(&mut self) {
        self.success_count += 1;
        
        match self.state {
            CircuitState::HalfOpen => {
                // Recovered! Close the circuit
                if self.success_count >= 3 {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    info!("🔓 Circuit breaker CLOSED (recovered)");
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            _ => {}
        }
    }

    /// Record a failed call
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(chrono::Utc::now().timestamp());
        
        if self.failure_count >= self.threshold {
            self.state = CircuitState::Open;
            warn!("🔒 Circuit breaker OPEN (too many failures: {})", self.failure_count);
        }
    }

    /// Check if we can make a call
    pub fn can_call(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = self.last_failure_time {
                    let now = chrono::Utc::now().timestamp();
                    if now - last_failure > self.timeout_secs as i64 {
                        // Try half-open
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        info!("🔓 Circuit breaker HALF-OPEN (testing recovery)");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// Exchange weight for weighted averaging
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExchangeWeight {
    pub exchange: String,
    pub weight: f64, // Based on volume/reliability
}

impl Default for ExchangeWeight {
    fn default() -> Self {
        Self {
            exchange: String::new(),
            weight: 1.0,
        }
    }
}

/// Price with metadata
#[derive(Clone, Debug)]
pub struct PriceData {
    pub price: f64,
    pub exchange: String,
    pub timestamp: i64,
}

/// Robust price fetcher
pub struct RobustFetcher {
    circuit_breakers: HashMap<String, CircuitBreaker>,
    exchange_weights: HashMap<String, f64>,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl RobustFetcher {
    pub fn new() -> Self {
        // Default exchange weights (based on volume/reliability)
        let mut weights = HashMap::new();
        weights.insert("binance".to_string(), 1.5);   // Highest volume
        weights.insert("coinbase".to_string(), 1.3);  // High reliability
        weights.insert("kraken".to_string(), 1.2);    // Good reliability
        weights.insert("okx".to_string(), 1.0);       // Standard
        weights.insert("bybit".to_string(), 1.0);     // Standard

        Self {
            circuit_breakers: HashMap::new(),
            exchange_weights: weights,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }

    /// Get or create circuit breaker for an exchange
    fn get_circuit_breaker(&mut self, exchange: &str) -> &mut CircuitBreaker {
        self.circuit_breakers
            .entry(exchange.to_string())
            .or_insert_with(|| CircuitBreaker::new(5, 60))
    }

    /// Fetch price with retry logic and circuit breaker
    pub async fn fetch_price_robust(
        &mut self,
        symbol: &str,
        exchange: &str,
    ) -> Result<f64> {
        let breaker = self.get_circuit_breaker(exchange);
        
        // Check circuit breaker
        if !breaker.can_call() {
            return Err(anyhow::anyhow!("Circuit breaker OPEN for {}", exchange));
        }

        // Retry logic with exponential backoff
        let mut retries = 0;
        let mut delay = Duration::from_millis(self.retry_delay_ms);

        loop {
            match self.fetch_price_once(symbol, exchange).await {
                Ok(price) => {
                    // Success!
                    let breaker = self.get_circuit_breaker(exchange);
                    breaker.record_success();
                    return Ok(price);
                }
                Err(e) if retries < self.max_retries => {
                    retries += 1;
                    warn!("⚠️  Retry {}/{} for {} on {}: {}", retries, self.max_retries, symbol, exchange, e);
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => {
                    // All retries failed
                    let breaker = self.get_circuit_breaker(exchange);
                    breaker.record_failure();
                    return Err(e);
                }
            }
        }
    }

    /// Single fetch attempt (implement actual API calls here)
    async fn fetch_price_once(&self, symbol: &str, exchange: &str) -> Result<f64> {
        // This would call the actual exchange API
        // For now, placeholder that calls existing fetch functions
        match exchange {
            "binance" => super::fetch_binance(symbol).await,
            "coinbase" => super::fetch_coinbase(symbol).await,
            "kraken" => super::fetch_kraken(symbol).await,
            "okx" => super::fetch_okx(symbol).await,
            "bybit" => super::fetch_bybit(symbol).await,
            _ => Err(anyhow::anyhow!("Unknown exchange: {}", exchange)),
        }
    }

    /// Fetch from multiple exchanges
    pub async fn fetch_from_exchanges(
        &mut self,
        symbol: &str,
        exchanges: &[String],
    ) -> Vec<PriceData> {
        let mut prices = Vec::new();
        let timestamp = chrono::Utc::now().timestamp();

        for exchange in exchanges {
            match self.fetch_price_robust(symbol, exchange).await {
                Ok(price) => {
                    prices.push(PriceData {
                        price,
                        exchange: exchange.clone(),
                        timestamp,
                    });
                }
                Err(e) => {
                    warn!("Failed to fetch {} from {}: {}", symbol, exchange, e);
                }
            }
        }

        prices
    }

    /// Remove outliers using standard deviation
    pub fn remove_outliers(&self, prices: &[PriceData]) -> Vec<PriceData> {
        if prices.len() < 3 {
            return prices.to_vec();
        }

        let values: Vec<f64> = prices.iter().map(|p| p.price).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        // Calculate standard deviation
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Keep prices within 3 standard deviations
        prices
            .iter()
            .filter(|p| (p.price - mean).abs() <= 3.0 * std_dev)
            .cloned()
            .collect()
    }

    /// Calculate weighted average
    pub fn weighted_average(&self, prices: &[PriceData]) -> Option<f64> {
        if prices.is_empty() {
            return None;
        }

        let mut total = 0.0;
        let mut weight_sum = 0.0;

        for price_data in prices {
            let weight = self.exchange_weights
                .get(&price_data.exchange)
                .copied()
                .unwrap_or(1.0);
            
            total += price_data.price * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            Some(total / weight_sum)
        } else {
            None
        }
    }

    /// Calculate median
    pub fn median(&self, prices: &[PriceData]) -> Option<f64> {
        if prices.is_empty() {
            return None;
        }

        let mut values: Vec<f64> = prices.iter().map(|p| p.price).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = values.len() / 2;
        if values.len() % 2 == 0 {
            Some((values[mid - 1] + values[mid]) / 2.0)
        } else {
            Some(values[mid])
        }
    }

    /// Calculate confidence (based on spread)
    pub fn confidence(&self, prices: &[PriceData]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let values: Vec<f64> = prices.iter().map(|p| p.price).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let max_deviation = values.iter()
            .map(|v| (v - mean).abs() / mean)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Confidence decreases with spread
        (1.0 - max_deviation).max(0.0)
    }

    /// Validate price is in reasonable range
    pub fn validate_price(&self, symbol: &str, price: f64) -> bool {
        if price <= 0.0 {
            return false;
        }

        // Symbol-specific validation
        match symbol {
            s if s.contains("BTC") => price > 1000.0 && price < 1_000_000.0,
            s if s.contains("ETH") => price > 10.0 && price < 100_000.0,
            s if s.contains("SOL") => price > 0.1 && price < 10_000.0,
            s if s.contains("XNT") => price > 0.0001 && price < 1000.0,
            _ => price > 0.0 && price < 1_000_000.0,
        }
    }

    /// Check if price data is stale
    pub fn is_stale(&self, timestamp: i64, max_age_secs: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - timestamp) > max_age_secs
    }

    /// Full robust aggregation pipeline
    pub async fn aggregate_price(
        &mut self,
        symbol: &str,
        exchanges: &[String],
    ) -> Result<(f64, f64)> {
        // 1. Fetch from all exchanges
        let mut prices = self.fetch_from_exchanges(symbol, exchanges).await;

        if prices.is_empty() {
            return Err(anyhow::anyhow!("No prices fetched for {}", symbol));
        }

        info!("📊 Fetched {} prices for {}", prices.len(), symbol);

        // 2. Remove outliers
        prices = self.remove_outliers(&prices);
        info!("📊 After outlier removal: {} prices", prices.len());

        if prices.is_empty() {
            return Err(anyhow::anyhow!("All prices were outliers for {}", symbol));
        }

        // 3. Validate prices
        prices.retain(|p| self.validate_price(symbol, p.price));
        info!("📊 After validation: {} prices", prices.len());

        if prices.is_empty() {
            return Err(anyhow::anyhow!("No valid prices for {}", symbol));
        }

        // 4. Check staleness
        prices.retain(|p| !self.is_stale(p.timestamp, 60));

        if prices.is_empty() {
            return Err(anyhow::anyhow!("All prices are stale for {}", symbol));
        }

        // 5. Calculate weighted average
        let price = self.weighted_average(&prices)
            .ok_or_else(|| anyhow::anyhow!("Failed to calculate weighted average"))?;

        // 6. Calculate confidence
        let confidence = self.confidence(&prices);

        info!("✅ Aggregated price for {}: ${:.2} (confidence: {:.2}%)", symbol, price, confidence * 100.0);

        Ok((price, confidence))
    }
}

impl Default for RobustFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new(3, 60);
        assert_eq!(breaker.state, CircuitState::Closed);

        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state, CircuitState::Closed);

        breaker.record_failure();
        assert_eq!(breaker.state, CircuitState::Open);
    }

    #[test]
    fn test_remove_outliers() {
        let fetcher = RobustFetcher::new();
        
        let prices = vec![
            PriceData { price: 100.0, exchange: "a".to_string(), timestamp: 1000 },
            PriceData { price: 101.0, exchange: "b".to_string(), timestamp: 1000 },
            PriceData { price: 102.0, exchange: "c".to_string(), timestamp: 1000 },
            PriceData { price: 200.0, exchange: "d".to_string(), timestamp: 1000 }, // Outlier
        ];

        let filtered = fetcher.remove_outliers(&prices);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_weighted_average() {
        let mut fetcher = RobustFetcher::new();
        fetcher.exchange_weights.insert("high".to_string(), 2.0);
        fetcher.exchange_weights.insert("low".to_string(), 1.0);

        let prices = vec![
            PriceData { price: 100.0, exchange: "high".to_string(), timestamp: 1000 },
            PriceData { price: 110.0, exchange: "low".to_string(), timestamp: 1000 },
        ];

        let avg = fetcher.weighted_average(&prices).unwrap();
        // (100*2 + 110*1) / (2+1) = 310/3 = 103.33
        assert!((avg - 103.33).abs() < 0.01);
    }

    #[test]
    fn test_median() {
        let fetcher = RobustFetcher::new();
        
        let prices = vec![
            PriceData { price: 100.0, exchange: "a".to_string(), timestamp: 1000 },
            PriceData { price: 102.0, exchange: "b".to_string(), timestamp: 1000 },
            PriceData { price: 101.0, exchange: "c".to_string(), timestamp: 1000 },
        ];

        let median = fetcher.median(&prices).unwrap();
        assert_eq!(median, 101.0);
    }

    #[test]
    fn test_validate_price() {
        let fetcher = RobustFetcher::new();
        
        assert!(fetcher.validate_price("BTC/USD", 50000.0));
        assert!(!fetcher.validate_price("BTC/USD", 100.0)); // Too low
        assert!(!fetcher.validate_price("BTC/USD", 2_000_000.0)); // Too high
        assert!(!fetcher.validate_price("BTC/USD", -100.0)); // Negative
    }

    #[test]
    fn test_confidence() {
        let fetcher = RobustFetcher::new();
        
        // Tight spread = high confidence
        let prices1 = vec![
            PriceData { price: 100.0, exchange: "a".to_string(), timestamp: 1000 },
            PriceData { price: 101.0, exchange: "b".to_string(), timestamp: 1000 },
        ];
        let conf1 = fetcher.confidence(&prices1);
        assert!(conf1 > 0.99);

        // Wide spread = low confidence
        let prices2 = vec![
            PriceData { price: 100.0, exchange: "a".to_string(), timestamp: 1000 },
            PriceData { price: 150.0, exchange: "b".to_string(), timestamp: 1000 },
        ];
        let conf2 = fetcher.confidence(&prices2);
        assert!(conf2 < 0.8);
    }
}

