# Deployment Guide for New Publishers

This guide will help you deploy a new Tachyon Oracle node and join the network as a publisher.

## 🎯 Overview

Each oracle node consists of:
- **Signer**: Fetches prices from exchanges and signs them
- **Relayer**: Aggregates signatures and submits to blockchain
- **API Service**: Provides monitoring and management interface

Multiple nodes work together to provide decentralized price feeds.

## 📋 Requirements

### Server Specifications
- **OS**: Ubuntu 20.04+ or Debian 11+
- **RAM**: 2GB minimum, 4GB recommended
- **Storage**: 20GB minimum
- **Network**: Stable internet connection
- **Ports**: 7777 (relayer), 7171 (API)

### Software (Auto-installed)
- Node.js 18+
- PM2 process manager
- Solana CLI tools
- Git

## 🚀 Step-by-Step Deployment

### Step 1: Prepare Your Server

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install git (if not already installed)
sudo apt install git -y
```

### Step 2: Clone Repository

```bash
# Clone the repository
git clone https://github.com/Xenian84/tachyon-oracles.git
cd tachyon-oracles
```

### Step 3: Run Setup Wizard

```bash
# Make the console executable
chmod +x tachyon

# Run the console
./tachyon
```

### Step 4: First Time Setup

In the console menu, select:

```
8. 🚀 First Time Setup Wizard
```

The wizard will guide you through:

1. **Dependency Installation** - Installs Node.js, PM2, Solana CLI
2. **Keypair Generation** - Creates unique keypairs for your node
3. **Environment Configuration** - Sets up RPC endpoints and ports
4. **Service Build** - Compiles TypeScript code
5. **Publisher Registration** - Registers your node on-chain
6. **Service Startup** - Starts signer, relayer, and API services
7. **API Service Setup** - Generates secure API key

### Step 5: Verify Deployment

```bash
# Check all services are running
pm2 list
```

Expected output:
```
┌─────┬──────────────────┬─────────┬─────────┬──────┐
│ id  │ name             │ status  │ restart │ cpu  │
├─────┼──────────────────┼─────────┼─────────┼──────┤
│ 0   │ tachyon-signer   │ online  │ 0       │ 5%   │
│ 1   │ tachyon-relayer  │ online  │ 0       │ 3%   │
│ 2   │ tachyon-api      │ online  │ 0       │ 1%   │
└─────┴──────────────────┴─────────┴─────────┴──────┘
```

### Step 6: Configure Firewall

```bash
# Allow API access (for Telegram bot)
sudo ufw allow 7171/tcp

# Allow relayer communication
sudo ufw allow 7777/tcp

# Keep SSH open!
sudo ufw allow 22/tcp

# Enable firewall
sudo ufw enable
```

### Step 7: Setup Remote Monitoring (Optional)

1. Open Telegram and search for `@tachyon_oracle_bot`
2. Send `/start` to the bot
3. Tap: 🗂️ Profiles → ➕ Add Profile
4. Enter your server details:
   - **Name**: Your server name (e.g., "Server 2")
   - **API URL**: `http://YOUR_SERVER_IP:7171`
   - **API Key**: Copy from setup wizard output
5. Bot will test the connection
6. You can now monitor your node remotely!

## 📊 Post-Deployment Checks

### 1. Verify Publisher Registration

```bash
./tachyon
# Select: 5. View Publishers
```

You should see your publisher address in the list.

### 2. Check Price Submissions

```bash
pm2 logs tachyon-relayer --lines 50
```

Look for:
- ✅ "Received message from [your-publisher]"
- ✅ "Submitting bundle for [asset-id]"

If you see "Insufficient publishers for quorum":
- This is normal if there are fewer than 3 publishers
- Prices will start appearing once 3+ publishers are active

## 🔧 Configuration

### Customize Price Sources

Edit `signer/config.yaml`:

```yaml
assets:
  - id: "BTC/USD"
    symbol:
      binance: "BTCUSDT"
      coinbase: "BTC-USD"
      kraken: "XBTUSD"
    sources: ["binance", "coinbase"]  # Remove/add sources
```

Restart signer after changes:
```bash
pm2 restart tachyon-signer
```

## 🛡️ Security Best Practices

### 1. Secure Your Private Keys

```bash
# Backup keys directory
tar -czf keys-backup.tar.gz keys/

# Store backup securely offline
# NEVER share or commit keys to git
```

### 2. Restrict API Access

Edit `api-service/.env`:
```bash
# Whitelist specific IPs
ALLOWED_IPS=YOUR_IP,TELEGRAM_BOT_IP

# Or allow all (less secure)
ALLOWED_IPS=0.0.0.0/0
```

## 📈 Monitoring

### Console Monitoring (Local)

```bash
./tachyon
```

Available options:
- 1. Service Manager - Start/stop/restart services
- 2. View Status - Network and service status
- 6. View Feeds - Current price feeds
- 5. View Publishers - All registered publishers
- 7. View Logs - Service logs

### Telegram Bot (Remote)

Commands:
- `/status` - Service and network status
- `/feeds` - View all price feeds
- `/publishers` - List publishers

## ❌ Troubleshooting

### Service Won't Start

```bash
# Check logs for errors
pm2 logs tachyon-signer --err

# Check if port is in use
sudo lsof -i :7777

# Restart with fresh state
pm2 delete all
pm2 start ecosystem.config.js
```

### "Insufficient Publishers" Error

This is normal when there are fewer than 3 publishers in the network.

**Solution**: Wait for more publishers to join, or contact the network admin.

### API Connection Failed

```bash
# Check if API is running
pm2 list

# Check API logs
pm2 logs tachyon-api

# Test API locally
curl http://localhost:7171/api/status
```

## ✅ Deployment Checklist

- [ ] Server meets minimum requirements
- [ ] Repository cloned
- [ ] Setup wizard completed successfully
- [ ] All services running (pm2 list shows 3 online)
- [ ] Publisher registered on-chain
- [ ] Firewall configured
- [ ] Keys backed up securely
- [ ] Telegram bot configured (optional)
- [ ] Monitoring working
- [ ] Logs showing price updates

## 🎉 Success!

Your oracle node is now part of the Tachyon network!

Once 3+ publishers are active, price feeds will start appearing on-chain and your node will be contributing to the decentralized oracle network.

---

**Need help?** Check [CONSOLE_GUIDE.md](CONSOLE_GUIDE.md) or open an issue on GitHub.
