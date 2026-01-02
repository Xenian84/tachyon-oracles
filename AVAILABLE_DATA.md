# 📊 Available Data for Tachyon Insights Dashboard

## ✅ YES - We Have This Data!

### 1. **Governance Contract Data** (TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9)

#### Global Network Stats:
```rust
pub struct GovernanceState {
    ✅ total_staked: u64              // Total TACH staked in network
    ✅ total_rewards_distributed: u64 // Lifetime rewards paid out
    ✅ total_stakers: u64             // Number of active validators
    ✅ total_proposals: u64           // Governance proposals count
    ✅ daily_rewards_rate: u64        // 82K TACH/day
    ✅ total_slashed: u64             // Total slashed tokens
    ✅ min_stake: u64                 // Minimum stake requirement
    ✅ rewards_paused: bool           // Rewards system status
    ✅ last_epoch_distribution: i64   // Last distribution time
}
```

**Available via:**
- Query governance state account
- RPC: `getAccountInfo(TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9)`

---

#### Per-Validator Stats:
```rust
pub struct StakerInfo {
    ✅ staked_amount: u64             // Individual stake
    ✅ total_rewards_claimed: u64     // Lifetime earnings
    ✅ pending_rewards: u64           // Unclaimed rewards
    ✅ compounded_rewards: u64        // Auto-staked rewards
    ✅ uptime_score: u64              // 0-10000 (0-100%)
    ✅ submissions_count: u64         // Total submissions
    ✅ accurate_submissions: u64      // Successful submissions
    ✅ first_stake_timestamp: i64     // When they joined
    ✅ last_stake_timestamp: i64      // Last stake time
    ✅ last_claim_timestamp: i64      // Last reward claim
    ✅ loyalty_tier: u8               // 0-4 (None to Platinum)
    ✅ referral_count: u64            // Referrals made
    ✅ referral_rewards: u64          // Referral earnings
    ✅ vested_rewards: u64            // Vested amount
}
```

**Available via:**
- Query each validator's staker PDA
- PDA: `[b"staker-v2", validator_pubkey]`

---

### 2. **L2 State Compression Data** (L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx)

```rust
pub struct L2State {
    ✅ current_root: [u8; 32]        // Current Merkle root
    ✅ batch_number: u64             // Batch counter (incrementing)
    ✅ feed_count: u32               // Number of feeds in batch
    ✅ last_update: i64              // Last submission timestamp
    ✅ authority: Pubkey             // Current authority
}
```

**Available via:**
- Query L2 state account
- RPC: `getAccountInfo(L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx)`

---

### 3. **On-Chain Transaction History**

✅ **Available via Solana RPC:**
- `getSignaturesForAddress()` - All transactions
- `getTransaction()` - Transaction details
- Filter by program ID to get:
  - Stake events
  - Unstake events
  - Reward claims
  - Batch submissions
  - Governance votes

---

### 4. **Token Data**

✅ **TACH Token Info:**
- Total supply
- Circulating supply
- Token holders
- Vault balance
- Rewards pool balance

**Available via:**
- SPL Token RPC calls
- `getTokenSupply()`
- `getTokenAccountBalance()`

---

## 🚧 MISSING - Need to Add

### 1. **Price Feed Data** ❌

**What we DON'T have:**
- Individual price feed values (BTC/USD, ETH/USD, etc.)
- Price history
- Feed metadata (symbol, description)
- Confidence intervals
- Publisher contributions per feed

**Why:**
- Current implementation only stores Merkle root
- Actual price data is not stored on-chain
- Need to add price feed storage or off-chain indexer

**Solutions:**
1. **Option A: Store prices on-chain**
   - Add `PriceFeed` account structure
   - Store latest price per feed
   - More expensive (storage costs)

2. **Option B: Off-chain indexer** ⭐ (Recommended)
   - Node submits to both on-chain + API
   - API stores full price history
   - Cheaper, more flexible
   - Can store unlimited history

3. **Option C: Event logs**
   - Emit events with price data
   - Index events off-chain
   - Good middle ground

---

### 2. **Historical Analytics** ❌

**What we DON'T have:**
- Historical stake amounts
- Performance over time
- Reward distribution history
- Network growth metrics

**Why:**
- Blockchain only stores current state
- No time-series data

**Solution:**
- **Indexer/Database** (PostgreSQL)
- Periodically snapshot on-chain data
- Store in time-series database
- Build historical charts

---

### 3. **Real-time Updates** ❌

**What we DON'T have:**
- WebSocket feed
- Real-time notifications
- Live price updates

**Solution:**
- **WebSocket Server**
- Poll blockchain every 1-5 seconds
- Push updates to connected clients
- Or use Solana's `onAccountChange` subscription

---

## 📋 Data We CAN Display Right Now

### ✅ Dashboard Overview Page
```
Total Staked:              ✅ (from GovernanceState.total_staked)
Active Validators:         ✅ (from GovernanceState.total_stakers)
Total Rewards Distributed: ✅ (from GovernanceState.total_rewards_distributed)
Current Batch Number:      ✅ (from L2State.batch_number)
Last Update:               ✅ (from L2State.last_update)
Daily Rewards Rate:        ✅ (from GovernanceState.daily_rewards_rate)
```

### ✅ Publishers/Validators Page
```
For each validator:
- Address                  ✅ (pubkey)
- Staked Amount            ✅ (StakerInfo.staked_amount)
- Uptime Score             ✅ (StakerInfo.uptime_score)
- Total Submissions        ✅ (StakerInfo.submissions_count)
- Accurate Submissions     ✅ (StakerInfo.accurate_submissions)
- Success Rate             ✅ (calculated: accurate/total)
- Total Rewards            ✅ (StakerInfo.total_rewards_claimed)
- Pending Rewards          ✅ (StakerInfo.pending_rewards)
- Loyalty Tier             ✅ (StakerInfo.loyalty_tier)
- Active Since             ✅ (StakerInfo.first_stake_timestamp)
- Referrals                ✅ (StakerInfo.referral_count)
```

### ✅ Network Status Page
```
Current Merkle Root:       ✅ (L2State.current_root)
Batch Number:              ✅ (L2State.batch_number)
Feeds in Batch:            ✅ (L2State.feed_count)
Last Update:               ✅ (L2State.last_update)
Total Stake:               ✅ (GovernanceState.total_staked)
Active Validators:         ✅ (GovernanceState.total_stakers)
```

### ✅ Individual Validator Page
```
All StakerInfo fields      ✅
Transaction history        ✅ (via getSignaturesForAddress)
Stake events               ✅ (via transaction parsing)
Reward claims              ✅ (via transaction parsing)
Performance chart          ⚠️ (need historical data)
```

### ❌ Price Feeds Page (MISSING DATA)
```
BTC/USD price              ❌ (not stored on-chain)
ETH/USD price              ❌ (not stored on-chain)
Price history              ❌ (not stored on-chain)
Confidence intervals       ❌ (not stored on-chain)
```

---

## 🎯 Recommended Architecture

### Phase 1: Basic Dashboard (Can Build NOW)
```
┌─────────────────────────────────────────┐
│         Next.js Frontend                │
│                                         │
│  - Overview page                        │
│  - Validators list                      │
│  - Network status                       │
│  - Individual validator pages           │
└─────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│         Solana RPC                      │
│                                         │
│  - Query GovernanceState                │
│  - Query StakerInfo PDAs                │
│  - Query L2State                        │
│  - Query transactions                   │
└─────────────────────────────────────────┘
```

**What works:**
✅ All validator stats
✅ Network stats
✅ Staking info
✅ Rewards tracking
✅ Performance metrics

**What's missing:**
❌ Price feeds
❌ Historical charts
❌ Real-time updates

---

### Phase 2: Full Dashboard (Need Indexer)
```
┌─────────────────────────────────────────┐
│         Next.js Frontend                │
│  + WebSocket for real-time              │
└─────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│         Backend API                     │
│                                         │
│  - REST endpoints                       │
│  - WebSocket server                     │
│  - Historical data                      │
└─────────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        ▼                 ▼
┌──────────────┐  ┌──────────────┐
│ Solana RPC   │  │ PostgreSQL   │
│              │  │              │
│ Current data │  │ Historical   │
└──────────────┘  └──────────────┘
        ▲                 ▲
        │                 │
        └─────────────────┘
              Indexer
    (Polls chain every 5s)
```

**Adds:**
✅ Historical charts
✅ Price feed storage
✅ Real-time updates
✅ Advanced analytics

---

## 🚀 Quick Start: What to Build First

### Week 1-2: MVP Dashboard (Using Only On-Chain Data)

**Pages:**
1. **Overview** - Network stats from GovernanceState + L2State
2. **Validators** - List all validators with performance metrics
3. **Validator Detail** - Individual validator page
4. **Network** - L2 state, batch info, recent submissions

**Tech Stack:**
- Next.js 14
- Tailwind CSS
- @solana/web3.js
- SWR for data fetching
- Recharts for simple charts

**No backend needed!** - Query RPC directly from frontend

---

### Week 3-4: Add Indexer & Historical Data

**Add:**
1. Node.js backend
2. PostgreSQL database
3. Indexer service (polls chain every 5s)
4. REST API
5. Historical charts

---

### Week 5-6: Add Price Feeds

**Options:**
1. Modify node to POST prices to API
2. Add price feed storage contract
3. Index price data from events

---

## 📝 Summary

### ✅ Can Build Right Now (No Changes Needed):
- Network overview dashboard
- Validator rankings
- Performance metrics
- Staking stats
- Rewards tracking
- Transaction history

### 🔧 Need to Add:
- Price feed storage (on-chain or off-chain)
- Historical data indexer
- WebSocket for real-time updates
- Time-series database

### 🎯 Recommendation:
**Start with Phase 1** - Build the dashboard using only on-chain data. This gives you:
- Professional-looking site
- All validator/network stats
- Real data from your contracts
- No backend complexity

Then add indexer + price feeds in Phase 2.

---

## 🔗 Example Queries

### Get Network Stats:
```typescript
const governanceState = await connection.getAccountInfo(
  new PublicKey("TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9")
);
// Parse GovernanceState struct
```

### Get All Validators:
```typescript
const validators = await connection.getProgramAccounts(
  governanceProgram,
  {
    filters: [
      { dataSize: 171 }, // StakerInfo size
      { memcmp: { offset: 0, bytes: "staker-v2" } }
    ]
  }
);
```

### Get L2 State:
```typescript
const l2State = await connection.getAccountInfo(
  new PublicKey("L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx")
);
// Parse L2State struct
```

---

**Ready to start building?** 🚀

