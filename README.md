# 🚀 Tachyon Oracle Network

**Decentralized Price Feeds for the X1 Blockchain**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Solana](https://img.shields.io/badge/solana-1.18%2B-blue.svg)](https://solana.com/)

Tachyon is a high-performance oracle network that provides real-time, decentralized price feeds for the X1 blockchain ecosystem. Built with Rust and powered by stake-weighted consensus, Tachyon ensures accurate and reliable data for DeFi applications.

---

## ✨ Features

- 🔄 **Real-Time Price Feeds** - Updates every 10 seconds from multiple exchanges
- 🔐 **Stake-Weighted Consensus** - Validators stake TACH tokens for network security
- 🌳 **Merkle Tree Compression** - Efficient on-chain data storage
- 📊 **Multiple Data Sources** - Aggregates from Coinbase, Kraken, and more
- ⚡ **Low Latency** - Sub-second price updates
- 🎯 **High Accuracy** - Confidence intervals and outlier detection
- 🔌 **Easy Integration** - Simple API for DeFi protocols

---

## 🚀 Quick Start (One-Click Install)

**For validators who want to run a node:**

```bash
curl -sSL https://raw.githubusercontent.com/xenian84/tachyon-oracles/main/install.sh | bash
```

That's it! The script will:
- ✅ Install all dependencies (Rust, Solana CLI, etc.)
- ✅ Build the node from source
- ✅ Generate a keypair
- ✅ Create configuration files
- ✅ Set up systemd service
- ✅ Start the node

**Total time: ~10 minutes** ⏱️

---

## 📋 Requirements

### Minimum:
- **OS:** Ubuntu 20.04+ or Debian 11+
- **CPU:** 2 cores
- **RAM:** 4 GB
- **Disk:** 20 GB SSD
- **Network:** 10 Mbps up/down

### Recommended:
- **OS:** Ubuntu 22.04 LTS
- **CPU:** 4+ cores
- **RAM:** 8+ GB
- **Disk:** 50+ GB NVMe SSD
- **Network:** 100+ Mbps up/down

### Tokens:
- **Stake:** 100,000+ TACH (minimum to participate)
- **Fees:** 0.1+ XNT (for transaction fees)

---

## 🏗️ Architecture

```
╔══════════════════════════════════════════════════════════════════╗
║                     TACHYON ORACLE NETWORK                       ║
╚══════════════════════════════════════════════════════════════════╝

    📊 Exchanges                    🌐 Oracle Nodes
    ┌─────────────┐                ┌──────────────┐
    │  Coinbase   │───────────────▶│   Node 1     │
    │   Kraken    │                │   Node 2     │
    │  Binance    │                │   Node 3     │
    └─────────────┘                │     ...      │
         │                         └──────┬───────┘
         │                                │
         │  Fetch Prices                  │  Aggregate
         │  Every 10s                     │  & Consensus
         │                                │
         ▼                                ▼
    ┌─────────────────────────────────────────────┐
    │          Local Aggregation                  │
    │   • Median calculation                      │
    │   • Confidence intervals                    │
    │   • Outlier detection                       │
    └─────────────────┬───────────────────────────┘
                      │
                      │  Submit Every 60s
                      │
         ┌────────────┴────────────┐
         │                         │
         ▼                         ▼
    ┌─────────────┐         ┌─────────────┐
    │  L2 State   │         │ Price Feeds │
    │ Compression │         │  Contract   │
    │             │         │             │
    │ Merkle Root │         │ Individual  │
    │  Storage    │         │   Prices    │
    └──────┬──────┘         └──────┬──────┘
           │                       │
           └───────────┬───────────┘
                       │
                       ▼
              ┌────────────────┐
              │  X1 Blockchain │
              │ (Solana-based) │
              └────────┬───────┘
                       │
                       ▼
              ┌────────────────┐
              │  DeFi Protocols│
              │   • DEXs       │
              │   • Lending    │
              │   • Derivatives│
              └────────────────┘
```

---

## 📦 What's Included

### 1. **Oracle Node** (`tachyon-node/`)
Rust-based oracle node that:
- Fetches prices from exchanges
- Aggregates data locally
- Participates in consensus
- Submits to blockchain

### 2. **Console** (`tachyon-console.sh`)
User-friendly management interface:
- Start/stop/restart node
- View logs and status
- Manage stake
- Check performance
- View rewards

### 3. **Smart Contracts** (Referenced, not deployed by nodes)
- **Governance:** Staking and validator management
- **L2 State Compression:** Merkle root storage
- **Price Feeds:** Individual price storage

---

## 🎮 Managing Your Node

After installation, use the console:

```bash
tachyon-console
```

You'll see a menu like this:

```
╔════════════════════════════════════════════════════════════╗
║           🚀 TACHYON ORACLE NODE CONSOLE 🚀               ║
╚════════════════════════════════════════════════════════════╝

[1] 🎮 Node Control
[2] 📊 View Logs
[3] 💰 Stake Management
[4] 📈 Performance Metrics
[5] 🎁 Rewards
[6] 💼 Wallet Info
[7] 🌐 Network Status
[8] ⚙️  Configuration
[9] ❌ Exit

Choose an option:
```

---

## 🔧 Manual Installation (Advanced)

If you prefer to install manually:

### 1. Install Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build tools
sudo apt install -y curl git build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### 2. Clone Repository

```bash
git clone https://github.com/xenian84/tachyon-oracles.git
cd tachyon-oracles
```

### 3. Build Node

```bash
cd tachyon-node
cargo build --release
```

### 4. Generate Keypair

```bash
solana-keygen new --outfile ~/.config/tachyon/node-keypair.json
```

### 5. Configure

```bash
cp config.example.toml ~/.config/tachyon/node-config.toml
# Edit config with your settings
```

### 6. Run Node

```bash
./target/release/tachyon-node start --config ~/.config/tachyon/node-config.toml
```

---

## 💰 Staking

To participate in consensus, you must stake TACH tokens:

### Minimum Stake: **100,000 TACH**

### Using the Console:
```bash
tachyon-console
# Select option [3] Stake Management
```

### Using CLI:
```bash
tachyon-node stake \
  --amount 100000 \
  --config ~/.config/tachyon/node-config.toml
```

---

## 📊 Monitoring

### Check Node Status:
```bash
systemctl status tachyon-node
```

### View Live Logs:
```bash
journalctl -u tachyon-node -f
```

### Check Stake:
```bash
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml
```

### Check Performance:
```bash
tachyon-node view-performance --config /etc/tachyon/node-config.toml
```

---

## 🌐 Network Information

### Mainnet:
- **RPC:** `https://rpc.mainnet.x1.xyz`
- **Explorer:** `https://explorer.x1.xyz`
- **Governance Program:** `TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9`
- **L2 Program:** `L2STdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9`
- **Price Feeds Program:** `PFEDu3nNzRQQYmX1Xvso2BxtPbUQaZEVoiLbXDy6U3W`

### Active Price Feeds:
- BTC/USD (updates every 10s)
- ETH/USD (updates every 10s)
- SOL/USD (updates every 10s)
- XNT/USD (updates every 10s)
- TACH/USD (updates every 10s)

---

## 🔐 Security

### Best Practices:
1. **Secure Your Keypair** - Never share your node keypair
2. **Use Firewall** - Only expose necessary ports
3. **Keep Updated** - Regularly update your node
4. **Monitor Logs** - Watch for suspicious activity
5. **Backup Config** - Keep backups of your configuration

### Recommended Firewall Rules:
```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow API (if public)
sudo ufw allow 8080/tcp

# Enable firewall
sudo ufw enable
```

---

## 📚 Documentation

- **Full Docs:** [docs.tachyon.xyz](https://docs.tachyon.xyz) *(coming soon)*
- **API Reference:** [api.tachyon.xyz](https://api.tachyon.xyz) *(coming soon)*
- **Discord:** [discord.gg/tachyon](https://discord.gg/tachyon) *(coming soon)*
- **Twitter:** [@TachyonOracle](https://twitter.com/TachyonOracle) *(coming soon)*

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🆘 Support

Need help? Here's how to get support:

1. **Check Logs:** `journalctl -u tachyon-node -f`
2. **Discord:** Join our community *(coming soon)*
3. **GitHub Issues:** [Report a bug](https://github.com/xenian84/tachyon-oracles/issues)
4. **Email:** support@tachyon.xyz *(coming soon)*

---

## 🎯 Roadmap

### Phase 1: Launch ✅
- [x] Core oracle node
- [x] Price feeds contract
- [x] Stake-weighted consensus
- [x] Initial price feeds (BTC, ETH, SOL)

### Phase 2: Expansion 🚧
- [ ] Public dashboard (Pyth-style)
- [ ] More price feeds (100+ assets)
- [ ] Multi-validator network (10+ nodes)
- [ ] Historical data API

### Phase 3: Decentralization 🔮
- [ ] DAO governance for feed curation
- [ ] Community-driven development
- [ ] Cross-chain integration
- [ ] Advanced analytics

---

## 💡 Why Tachyon?

### **Fast**
- Sub-second price updates
- 10-second fetch intervals
- 60-second on-chain submissions

### **Accurate**
- Multiple data sources
- Median aggregation
- Confidence intervals
- Outlier detection

### **Secure**
- Stake-weighted consensus
- Slashing for bad actors
- Merkle tree verification
- On-chain transparency

### **Decentralized**
- No single point of failure
- Community-driven governance
- Open-source codebase
- Permissionless participation

---

## 🌟 Join the Network!

Become a Tachyon validator today:

```bash
curl -sSL https://raw.githubusercontent.com/xenian84/tachyon-oracles/main/install.sh | bash
```

**Let's build the future of decentralized oracles together!** 🚀

---

*Built with ❤️ by the Tachyon community*
