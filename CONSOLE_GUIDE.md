# Tachyon Oracle Console Guide

Complete guide for managing your Tachyon Oracle node using the console interface.

## 🚀 Starting the Console

```bash
cd tachyon-oracles
./tachyon
```

## 📋 Main Menu

```
╔════════════════════════════════════════════════════════════════╗
║              🌊 TACHYON ORACLE MANAGEMENT CONSOLE              ║
╠════════════════════════════════════════════════════════════════╣
║  1. 🔧 Service Manager                                         ║
║  2. 📊 View Status                                             ║
║  3. 🔄 Initialize Oracle                                       ║
║  4. 📝 Register Publisher                                      ║
║  5. 👥 View Publishers                                         ║
║  6. 📈 View Feeds                                              ║
║  7. 📋 View Logs                                               ║
║  8. 🚀 First Time Setup Wizard                                 ║
║  9. ❌ Exit                                                    ║
╚════════════════════════════════════════════════════════════════╝
```

## 1️⃣ Service Manager

Manage all oracle services (signer, relayer, API).

### Options:
- **Start Services** - Start signer, relayer, and API
- **Stop Services** - Stop all services
- **Restart Services** - Restart all services
- **View Service Status** - Check if services are running
- **API Service Manager** - Manage API service specifically

### Usage:
```bash
./tachyon
# Select: 1. Service Manager
# Choose your action
```

### Common Tasks:

**Start all services:**
```
1. Service Manager → 1. Start All Services
```

**Restart after config change:**
```
1. Service Manager → 3. Restart All Services
```

**Check service status:**
```
1. Service Manager → 4. View Service Status
```

## 2️⃣ View Status

Check the current status of your oracle node.

### Shows:
- ✅ Service status (signer, relayer running/stopped)
- 📊 Network info (number of feeds and publishers)
- 🔗 RPC connection
- 📈 Feed count
- 👥 Publisher count

### Usage:
```bash
./tachyon
# Select: 2. View Status
```

### Example Output:
```
📊 Tachyon Oracle Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Services:
  Signer:  ✅ Running
  Relayer: ✅ Running

Network:
  Feeds:      9
  Publishers: 3

RPC: https://rpc.mainnet.x1.xyz
```

## 3️⃣ Initialize Oracle

Initialize the oracle smart contract on-chain.

### ⚠️ Important:
- Only needed ONCE for the entire network
- Requires admin/authority keypair
- Sets MIN_PUBLISHERS (minimum publishers for quorum)
- Usually already done - check with admin first

### Usage:
```bash
./tachyon
# Select: 3. Initialize Oracle
# Enter MIN_PUBLISHERS (e.g., 3)
```

### When to Use:
- Setting up a new oracle network
- Resetting the oracle (dev/test only)
- **NOT needed for joining existing network**

## 4️⃣ Register Publisher

Register your node as a publisher on the network.

### What it Does:
- Registers your publisher keypair on-chain
- Allows your node to submit price signatures
- Required before your node can participate

### Usage:
```bash
./tachyon
# Select: 4. Register Publisher
```

### Requirements:
- Publisher keypair exists (`keys/publisher.json`)
- Wallet has X1 tokens for transaction fee (~0.01 X1)
- Oracle contract is initialized

### After Registration:
- Your publisher address will be displayed
- Verify with option 5 (View Publishers)
- Start services to begin submitting prices

## 5️⃣ View Publishers

List all registered publishers on the network.

### Shows:
- Publisher public keys
- Publisher PDAs (Program Derived Addresses)
- Active/inactive status
- Total count

### Usage:
```bash
./tachyon
# Select: 5. View Publishers
```

### Example Output:
```
👥 Registered Publishers (3)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. ✅ Bqc3QJsDpXx91Yqn3HjPeMdDANbg5u4aPASGKcHik5h8
   PDA: F74jmJT24QVvKefUrUB6sfFbQRVRu97vLTBYyLsmr5D6

2. ✅ 7xKz9pRqYnM3vLbP2wQ8jNfT4sR6hC5dE1aF9bG3kH2m
   PDA: 8yLa0qSzPo4wNcMd3xJ7tU9vR1fK6gB4eN2pQ5mT8oH3

3. ✅ 3mN5pQ8rT2wV9xA4bC7dE1fG6hJ9kL2nM5oP8qR3sT6u
   PDA: 4nO6qR9sU3xW0yB5cD8eF2gH7jK0lM3nN6pQ9rS4tU7v
```

## 6️⃣ View Feeds

Display current price feeds from the oracle.

### Shows:
- Trading pairs (BTC/USD, ETH/USD, etc.)
- Current prices
- Confidence intervals
- Last update timestamps
- Staleness indicators

### Usage:
```bash
./tachyon
# Select: 6. View Feeds
```

### Example Output:
```
📈 Price Feeds (9)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

BTC/USD
  💰 $87,299.50 ±$3.51
  🕐 2025-12-26 20:45:30
  ✅ Fresh

ETH/USD
  💰 $2,919.96 ±$0.07
  🕐 2025-12-26 20:45:30
  ✅ Fresh

SOL/USD
  💰 $121.81 ±$0.00
  🕐 2025-12-26 20:45:30
  ✅ Fresh
```

### Status Indicators:
- ✅ Fresh - Updated within last minute
- ⚠️ Stale - Updated > 1 minute ago
- ❌ No data - Never updated

## 7️⃣ View Logs

View service logs for debugging and monitoring.

### Options:
- **Signer Logs** - Price fetching and signing activity
- **Relayer Logs** - Transaction submission logs
- **API Logs** - API service logs

### Usage:
```bash
./tachyon
# Select: 7. View Logs
# Choose service (signer/relayer/api)
```

### What to Look For:

**Signer Logs (Good):**
```
BTC/USD: price=87299.495000, conf=3.505000, sources=2
Submitted BTC/USD to http://localhost:7777
```

**Relayer Logs (Good):**
```
Received message from Bqc3QJ... for asset 7b4c9651...
Submitting bundle for 7b4c9651... with 3 publishers
✅ Bundle submitted successfully
```

**Relayer Logs (Problem):**
```
❌ Failed to submit bundle: Insufficient publishers for quorum
```
*Solution: Need more publishers (minimum 3)*

## 8️⃣ First Time Setup Wizard

Automated setup for new nodes. **Recommended for first deployment!**

### What it Does:
1. ✅ Installs dependencies (Node.js, PM2, Solana CLI)
2. ✅ Generates dedicated oracle keypairs:
   - `signer.json` - Signs price data (registered as publisher)
   - `relayer.json` - Submits transactions (pays gas fees)
   - ⚠️ **Note**: Separate from your validator keys!
3. ✅ Configures environment (.env file)
4. ✅ Builds SDK, signer, and relayer services
5. ✅ Registers signer key as publisher on-chain
6. ✅ Starts all services with PM2
7. ✅ Sets up API service for remote monitoring

### Usage:
```bash
./tachyon
# Select: 8. First Time Setup Wizard
# Follow the prompts
```

### Duration:
- Dependency installation: 2-5 minutes
- Keypair generation: < 1 minute
- Build (SDK + services): 2-3 minutes
- Total: ~5-10 minutes

### After Setup:
- All services running
- Publisher registered
- API service configured
- Ready to contribute to network

## 🔧 Common Tasks

### Starting Fresh Node

```bash
# 1. Clone repository
git clone https://github.com/Xenian84/tachyon-oracles
cd tachyon-oracles

# 2. Run setup wizard
./tachyon
# Select: 8. First Time Setup Wizard

# 3. Verify services
./tachyon
# Select: 2. View Status
```

### Checking if Everything is Working

```bash
./tachyon
# Select: 2. View Status
# Should show: Signer ✅ Running, Relayer ✅ Running

# Select: 5. View Publishers
# Should see your publisher in the list

# Select: 6. View Feeds
# Should see price feeds (if 3+ publishers active)
```

### Restarting After Config Change

```bash
# 1. Edit configuration
nano .env
# or
nano signer/config.yaml

# 2. Restart services
./tachyon
# Select: 1. Service Manager → 3. Restart All Services
```

### Debugging Issues

```bash
# 1. Check service status
./tachyon
# Select: 2. View Status

# 2. View logs
./tachyon
# Select: 7. View Logs → Choose service

# 3. Check for errors
# Look for ❌ or "error" in logs
```

## 📊 Monitoring

### Using PM2 Directly

```bash
# List all services
pm2 list

# Real-time logs
pm2 logs

# Monitor resources
pm2 monit

# Restart specific service
pm2 restart tachyon-signer
```

### Using Telegram Bot

For remote monitoring, set up the Telegram bot:

1. Open [@tachyon_oracle_bot](https://t.me/tachyon_oracle_bot)
2. Send `/start`
3. Add your server profile
4. Monitor remotely from anywhere!

## ⚙️ Configuration Files

### .env
Main environment configuration:
```bash
RPC_URL=https://rpc.mainnet.x1.xyz
RELAYER_PORT=7777
API_PORT=7171
```

### signer/config.yaml
Asset and price source configuration:
```yaml
assets:
  - id: "BTC/USD"
    symbol:
      binance: "BTCUSDT"
      coinbase: "BTC-USD"
    sources: ["binance", "coinbase"]
```

### api-service/.env
API service configuration:
```bash
API_PORT=7171
API_KEY=<generated>
API_MODE=monitoring
ALLOWED_IPS=127.0.0.1
```

## 🛠️ Troubleshooting

### Services Won't Start

```bash
# Check logs
./tachyon
# Select: 7. View Logs

# Check PM2
pm2 list

# Restart
./tachyon
# Select: 1. Service Manager → 3. Restart All Services
```

### "Insufficient Publishers" Error

**Cause:** Network has fewer than MIN_PUBLISHERS (usually 3)

**Solution:** Deploy more publisher nodes or wait for others to join

### Port Already in Use

```bash
# Check what's using the port
sudo lsof -i :7777

# Change port in .env
nano .env
# Edit RELAYER_PORT=7777 to different port

# Restart services
./tachyon
# Select: 1. Service Manager → 3. Restart All Services
```

### Can't Register Publisher

**Check:**
- Wallet has X1 tokens for transaction fee
- Oracle is initialized
- Publisher keypair exists (`keys/publisher.json`)
- RPC connection is working

## 📞 Getting Help

- **Console**: Type `9` to exit, then restart with `./tachyon`
- **Logs**: Always check logs first (option 7)
- **GitHub**: [Open an issue](https://github.com/Xenian84/tachyon-oracles/issues)
- **Telegram**: Join the community

## 🎯 Quick Reference

| Task | Menu Option |
|------|-------------|
| Start services | 1 → 1 |
| Stop services | 1 → 2 |
| Restart services | 1 → 3 |
| Check status | 2 |
| Register publisher | 4 |
| View publishers | 5 |
| View price feeds | 6 |
| View logs | 7 |
| First time setup | 8 |

---

**Need more help?** Check the [README.md](README.md) or open an issue on GitHub.

