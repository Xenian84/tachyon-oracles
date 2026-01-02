# 📊 Tachyon Price Feeds Contract

## Overview

The Price Feeds contract stores and manages real-time price data from oracle nodes, enabling the Tachyon Insights Dashboard to display live price feeds similar to Pyth Network.

---

## 🔑 Contract Details

**Program ID:** `PFEDu3nNzRQQYmX1Xvso2BxtPbUQaZEVoiLbXDy6U3W`

**Location:** `/l2-contracts/programs/tachyon-price-feeds/`

**Vanity Address:** Starts with "PFED" (Price FEeD)

---

## 📋 Features

### 1. **Individual Price Feeds**
Each trading pair (BTC/USD, ETH/USD, etc.) has its own on-chain account storing:
- Current price
- Confidence interval
- Last update timestamp
- Number of contributing publishers
- Feed status (Active/Inactive/Deprecated)

### 2. **Price Aggregation**
- Accepts multiple price submissions from different validators
- Calculates median price (resistant to outliers)
- Computes confidence interval (standard deviation)
- Stores aggregated result on-chain

### 3. **Event Emissions**
- `PriceUpdated` - Single price update
- `PriceAggregated` - Aggregated price from multiple sources
- Events can be indexed for historical data

### 4. **Access Control**
- Only authorized validators (staked in governance) can submit prices
- Authority can update feed status
- Public read access for all feeds

---

## 🏗️ Data Structures

### PriceFeed Account
```rust
pub struct PriceFeed {
    pub authority: Pubkey,        // Contract authority
    pub symbol: String,            // e.g., "BTC/USD" (max 32 chars)
    pub description: String,       // e.g., "Bitcoin to US Dollar" (max 128 chars)
    pub decimals: u8,              // Price decimals
    pub price: i64,                // Current price
    pub confidence: u64,           // Confidence interval
    pub expo: i32,                 // Price exponent (e.g., -8 for 8 decimals)
    pub last_update: i64,          // Last update timestamp
    pub publisher_count: u32,      // Number of publishers
    pub status: u8,                // 0=Inactive, 1=Active, 2=Deprecated
    pub bump: u8,                  // PDA bump
}
```

### PDA Seeds
```
seeds = [b"price-feed", symbol.as_bytes()]
```

Example PDAs:
- BTC/USD: `[b"price-feed", b"BTC/USD"]`
- ETH/USD: `[b"price-feed", b"ETH/USD"]`

---

## 🔧 Instructions

### 1. `initialize_feed`
Create a new price feed

**Parameters:**
- `symbol: String` - Trading pair (e.g., "BTC/USD")
- `description: String` - Human-readable description
- `decimals: u8` - Number of decimals

**Access:** Authority only

**Example:**
```typescript
await program.methods
  .initializeFeed("BTC/USD", "Bitcoin to US Dollar", 8)
  .accounts({
    priceFeed: priceFeedPDA,
    authority: authority.publicKey,
    payer: payer.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

---

### 2. `update_price`
Update a single price feed

**Parameters:**
- `price: i64` - Price value
- `confidence: u64` - Confidence interval
- `expo: i32` - Price exponent
- `publisher: Pubkey` - Publisher address

**Access:** Staked validators only

**Example:**
```typescript
await program.methods
  .updatePrice(
    new BN(97182386),  // $97,182.386
    new BN(132564),     // ±$132.564
    -8,                 // 8 decimals
    publisher.publicKey
  )
  .accounts({
    priceFeed: priceFeedPDA,
    submitter: validator.publicKey,
  })
  .rpc();
```

---

### 3. `aggregate_prices`
Aggregate multiple price submissions

**Parameters:**
- `prices: Vec<PriceSubmission>` - Array of price submissions

**Access:** Authority or validators

**Example:**
```typescript
const submissions = [
  { publisher: pub1, price: 97180000, confidence: 100000, expo: -8, timestamp: now },
  { publisher: pub2, price: 97185000, confidence: 120000, expo: -8, timestamp: now },
  { publisher: pub3, price: 97182000, confidence: 110000, expo: -8, timestamp: now },
];

await program.methods
  .aggregatePrices(submissions)
  .accounts({
    priceFeed: priceFeedPDA,
    authority: authority.publicKey,
  })
  .rpc();
```

**Algorithm:**
1. Sort all prices
2. Take median (middle value)
3. Calculate standard deviation for confidence
4. Store aggregated result

---

### 4. `get_price`
Query current price (view function)

**Returns:** `PriceData`

**Example:**
```typescript
const priceData = await program.methods
  .getPrice()
  .accounts({
    priceFeed: priceFeedPDA,
  })
  .view();

console.log(`${priceData.symbol}: $${priceData.price / 10**8}`);
```

---

### 5. `update_status`
Change feed status

**Parameters:**
- `status: u8` - 0=Inactive, 1=Active, 2=Deprecated

**Access:** Authority only

---

## 📡 Events

### PriceUpdated
```rust
#[event]
pub struct PriceUpdated {
    pub symbol: String,
    pub price: i64,
    pub confidence: u64,
    pub expo: i32,
    pub publisher: Pubkey,
    pub timestamp: i64,
}
```

### PriceAggregated
```rust
#[event]
pub struct PriceAggregated {
    pub symbol: String,
    pub price: i64,
    pub confidence: u64,
    pub publisher_count: u32,
    pub timestamp: i64,
}
```

**Use Case:** Indexers listen to these events to build historical price charts

---

## 🚀 Deployment

### Step 1: Build
```bash
cd l2-contracts
anchor build
```

### Step 2: Deploy
```bash
anchor deploy --provider.cluster https://rpc.mainnet.x1.xyz
```

### Step 3: Initialize Feeds
```bash
# Run initialization script
node scripts/init-price-feeds.js
```

---

## 📊 Example Feeds to Initialize

### Crypto
- BTC/USD - Bitcoin
- ETH/USD - Ethereum
- SOL/USD - Solana
- XNT/USD - X1 Native Token
- TACH/USD - Tachyon Token

### Forex
- EUR/USD - Euro to Dollar
- GBP/USD - Pound to Dollar
- JPY/USD - Yen to Dollar

### Commodities
- XAU/USD - Gold
- XAG/USD - Silver
- WTI/USD - Oil (WTI)

---

## 🔗 Integration with Tachyon Node

### Update Node to Submit Prices

The oracle node needs to be updated to submit price data after fetching from exchanges:

```rust
// In tachyon-node/src/sequencer/mod.rs

// After fetching prices from exchanges
for (symbol, price_data) in aggregated_prices {
    // Submit to price feeds contract
    let ix = update_price_instruction(
        &price_feed_program_id,
        &price_feed_pda,
        &node_keypair.pubkey(),
        price_data.price,
        price_data.confidence,
        price_data.expo,
        &node_keypair.pubkey(),
    );
    
    // Send transaction
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&node_keypair.pubkey()),
        &[&node_keypair],
        recent_blockhash,
    );
    
    rpc_client.send_and_confirm_transaction(&tx)?;
}
```

---

## 🎯 Dashboard Integration

### Query All Feeds
```typescript
// Get all price feed accounts
const feeds = await connection.getProgramAccounts(
  priceFeedProgramId,
  {
    filters: [
      { dataSize: PriceFeed.size },
    ]
  }
);

// Parse and display
for (const feed of feeds) {
  const data = PriceFeed.decode(feed.account.data);
  console.log(`${data.symbol}: $${data.price / 10**data.expo}`);
}
```

### Real-time Updates
```typescript
// Subscribe to account changes
connection.onAccountChange(
  priceFeedPDA,
  (accountInfo) => {
    const data = PriceFeed.decode(accountInfo.data);
    updateDashboard(data);
  }
);
```

### Historical Data
```typescript
// Listen to events
connection.onLogs(
  priceFeedProgramId,
  (logs) => {
    // Parse PriceUpdated events
    // Store in database for charts
  }
);
```

---

## 💡 Benefits vs Off-Chain Storage

### On-Chain (This Contract) ✅
**Pros:**
- Fully decentralized
- Verifiable on-chain
- No API dependency
- Trustless

**Cons:**
- Storage costs
- Limited history
- Transaction fees per update

### Off-Chain (API + Database)
**Pros:**
- Unlimited history
- Cheaper
- More flexible queries
- Faster updates

**Cons:**
- Centralized
- Requires API maintenance
- Trust in API operator

### Hybrid Approach ⭐ (Recommended)
- Store **latest prices** on-chain (this contract)
- Store **historical data** off-chain (indexer + PostgreSQL)
- Best of both worlds!

---

## 📈 Storage Costs

### Per Feed Account
- Size: ~300 bytes
- Rent: ~0.002 SOL (one-time)

### For 100 Feeds
- Total: ~0.2 SOL (~$20 at $100/SOL)
- Very affordable!

---

## 🔒 Security

### Access Control
1. Only staked validators can submit prices
2. Authority can pause/unpause feeds
3. Aggregation prevents single-point manipulation

### Price Validation
- Confidence intervals detect outliers
- Median calculation resists manipulation
- Timestamp verification prevents stale data

### Upgrade Authority
- Controlled by governance
- Multi-sig recommended
- Immutable after community approval

---

## 🎯 Next Steps

1. ✅ Contract created with vanity address
2. ⏳ Build and deploy contract
3. ⏳ Initialize common price feeds
4. ⏳ Update node to submit prices
5. ⏳ Build dashboard to display feeds
6. ⏳ Add indexer for historical data

---

## 📞 Contract Addresses

| Contract | Program ID |
|----------|-----------|
| **Price Feeds** | `PFEDu3nNzRQQYmX1Xvso2BxtPbUQaZEVoiLbXDy6U3W` |
| Governance | `TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9` |
| L2 State | `L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx` |

---

**Status:** ✅ Contract ready for deployment
**Next:** Build and deploy to X1 mainnet

