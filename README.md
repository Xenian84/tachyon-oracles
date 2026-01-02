# 🚀 Tachyon Oracle Node

A high-performance oracle node implementation in Rust for the X1 blockchain, providing decentralized price feeds with L2 state compression.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![X1 Blockchain](https://img.shields.io/badge/X1-Blockchain-blue.svg)](https://x1.xyz)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 📋 Overview

Tachyon Oracle Node is a Rust-based oracle implementation that:
- 🔄 Fetches real-time price data from multiple exchanges
- 🔐 Implements stake-weighted consensus for data validation
- 📦 Submits batched price feeds to X1 blockchain via L2 compression
- ⚡ Achieves high throughput with minimal on-chain footprint
- 🎯 Provides governance and staking mechanisms

## ✨ Features

- **Stake-Weighted Consensus**: Leader selection based on staked TACH tokens
- **L2 State Compression**: Efficient batch submission using Merkle trees
- **Multi-Exchange Support**: Aggregates data from Binance, Coinbase, and more
- **Governance System**: On-chain voting and proposal mechanisms
- **Performance Tracking**: Built-in metrics for uptime and accuracy
- **Rewards System**: Earn rewards for accurate price submissions
- **Easy Setup**: Automated installation script and comprehensive guide

## 🏗️ Architecture

```
╔═══════════════════════════════════════════════════════════════════╗
║                  🚀 Tachyon Oracle Node (Rust)                    ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║    ┌─────────────────┐      ┌─────────────────┐                   ║
║    │   📊 CONSENSUS         │  📦 SEQUENCER  │                   ║ 
║    │                 │      │                 │                   ║
║    │  Stake-Weighted │──────▶  Batch Price                       ║
║    │  Leader Select  │      │   Submissions   │                   ║
║    └─────────────────┘      └─────────────────┘                   ║
║            │                         │                            ║
║            │                         │                            ║
║            │                         ▼                            ║
║            │              ┌─────────────────┐                     ║
║            │              │ 🔗L2 CONTRACT  │                      
║            │              │                 │                     ║
║            │              │  Merkle Root    │                     ║
║            │              │    Storage      │                     ║
║            │              └─────────────────┘                     ║
║            │                         │                            ║
║            └─────────────────────────┘                            ║
║                          │                                        ║
║                          ▼                                        ║
║              ┌───────────────────────┐                            ║
║                ⚖️  GOVERNANCE                                    ║
║              │                       │                            ║
║              │  • Staking System     │                            ║
║              │  • Rewards Pool       │                            ║
║              │  • DAO Voting         │                            ║
║              │  • Performance Track  │                            ║
║              └───────────────────────┘                            ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝

Flow:
  1. Consensus module selects leader based on stake weight
  2. Sequencer batches price feeds from multiple exchanges
  3. Leader submits Merkle root to L2 Contract
  4. Governance tracks performance and distributes rewards
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70 or higher
- **Solana CLI** (for X1 blockchain)
- **Node.js** 16+ (for setup scripts)
- **100,000+ TACH tokens** for staking
- **~0.1 XNT** for transaction fees

### Installation

```bash
# Clone the repository
git clone https://github.com/xenian84/tachyon-oracles.git
cd tachyon-oracles

# Run the automated setup script
bash setup-new-node.sh
```

The setup script will:
1. ✅ Create a new keypair for your node
2. ✅ Build the Rust node binary
3. ✅ Create configuration files
4. ✅ Stake your TACH tokens
5. ✅ Set up systemd service
6. ✅ Start the node

### Manual Setup

For detailed manual setup instructions, see [NEW_NODE_SETUP.md](NEW_NODE_SETUP.md).

## 📊 Minimum Requirements

| Requirement | Amount |
|------------|--------|
| **TACH Stake** | 100,000 TACH (minimum) |
| **XNT Balance** | 0.1 XNT (for fees) |
| **RAM** | 2GB minimum |
| **Disk** | 10GB available |
| **Network** | Stable internet connection |

## 🔧 Configuration

### Node Configuration

Edit `/etc/tachyon-node/node-config.toml`:

```toml
keypair_path = "/var/lib/tachyon/node-keypair.json"
rpc_url = "https://rpc.mainnet.x1.xyz"
program_id = "TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"
l2_program_id = "L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"
gossip_port = 9000
api_port = 7777
update_interval_ms = 1000
batch_interval_ms = 100
min_publishers = 3

[[assets]]
symbol = "BTC/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "ETH/USD"
exchanges = ["binance", "coinbase"]

# Add more assets as needed
```

## 📝 Usage

### 🎮 Interactive Console (Recommended)

The easiest way to manage your node is using the interactive console:

```bash
# Run the console
sudo bash tachyon-console.sh
```

**Console Features:**
- ✅ Node control (start/stop/restart)
- ✅ Real-time log viewing
- ✅ Stake management
- ✅ Performance metrics
- ✅ Reward claiming
- ✅ Wallet information
- ✅ Network status
- ✅ System health monitoring

### Service Management (Manual)

```bash
# Start the node
sudo systemctl start tachyon-node

# Stop the node
sudo systemctl stop tachyon-node

# Check status
sudo systemctl status tachyon-node

# View logs
sudo journalctl -u tachyon-node -f
```

### CLI Commands (Manual)

```bash
# View stake information
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml

# View performance metrics
tachyon-node view-performance --config /etc/tachyon/node-config.toml

# Claim rewards
tachyon-node claim-rewards --config /etc/tachyon/node-config.toml

# Stake additional TACH
tachyon-node stake --config /etc/tachyon/node-config.toml --amount 50000

# Unstake TACH
tachyon-node unstake --config /etc/tachyon/node-config.toml --amount 50000
```

## 📈 Monitoring

### Check Node Status

```bash
# View detailed stake information
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml
```

**Output:**
```
╔══════════════════════════════════════════════════════════════╗
║              📊 DETAILED STAKE INFORMATION                   ║
╠══════════════════════════════════════════════════════════════╣
║ 💰 Staked Amount:        100000000.00 TACH                   ║
║ 📅 Staked Since:         2026-01-02 12:20                    ║
╠══════════════════════════════════════════════════════════════╣
║                    🎁 REWARDS SUMMARY                        ║
╠══════════════════════════════════════════════════════════════╣
║ 💎 Pending Rewards:              0.00 TACH                   ║
║ ✅ Total Claimed:                0.00 TACH                   ║
║ 🔄 Compounded:                   0.00 TACH                   ║
╠══════════════════════════════════════════════════════════════╣
║                  📈 PERFORMANCE METRICS                      ║
╠══════════════════════════════════════════════════════════════╣
║ 🎯 Uptime Score:         100% (1.5x multiplier)              ║
║ 📊 Submissions:                     0 total                  ║
║ ✅ Success Rate:           0% (0/0)                          ║
╚══════════════════════════════════════════════════════════════╝
```

### Performance Metrics

```bash
tachyon-node view-performance --config /etc/tachyon/node-config.toml
```

### Check Batch Submissions

```bash
# View logs for batch submissions
sudo journalctl -u tachyon-node -f | grep -i "submit"
```

## 🏆 Rewards System

### How Rewards Work

- **Daily Distribution**: 82,000 TACH distributed daily to all stakers
- **Proportional**: Rewards based on your stake percentage
- **Performance Multipliers**:
  - 100% uptime: 1.5x multiplier
  - 95-99% uptime: 1.25x multiplier
  - 90-94% uptime: 1.0x multiplier
  - Below 90%: 0.5x multiplier

### Claiming Rewards

```bash
tachyon-node claim-rewards --config /etc/tachyon/node-config.toml
```

## 🔐 Security

### Keypair Management

⚠️ **CRITICAL**: Your node keypair is stored at `/var/lib/tachyon/node-keypair.json`

**Security Best Practices:**
- ✅ Back up your keypair securely offline
- ✅ Never share your private key
- ✅ Keep keypair permissions at 600 (read/write for owner only)
- ✅ Use a separate keypair for each node
- ❌ Never commit keypairs to version control

### Firewall Configuration

```bash
# Allow gossip port (if running multiple nodes)
sudo ufw allow 9000/tcp

# Allow API port (if exposing API)
sudo ufw allow 7777/tcp

# Enable firewall
sudo ufw enable
```

## 🛠️ Troubleshooting

### Node Won't Start

```bash
# Check logs for errors
sudo journalctl -u tachyon-node -n 50

# Verify configuration
cat /etc/tachyon/node-config.toml

# Check keypair permissions
ls -la /var/lib/tachyon/node-keypair.json
```

### Not Submitting Batches

```bash
# Check if node is leader
sudo journalctl -u tachyon-node -f | grep -i "leader"

# Verify stake
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml

# Check for errors
sudo journalctl -u tachyon-node -p err
```

### Insufficient Funds

```bash
# Check XNT balance
solana balance /var/lib/tachyon/node-keypair.json --url https://rpc.mainnet.x1.xyz

# Transfer more XNT if needed
solana transfer \
  --url https://rpc.mainnet.x1.xyz \
  --keypair ~/.config/solana/id.json \
  <NODE_PUBKEY> \
  0.1
```

## 📚 Documentation

- **[NEW_NODE_SETUP.md](NEW_NODE_SETUP.md)** - Complete setup guide
- **[tachyon-node/README.md](tachyon-node/README.md)** - Node implementation details
- **[l2-contracts/](l2-contracts/)** - Smart contract documentation

## 🏗️ Smart Contracts

### ⚠️ Important: Contracts Are Already Deployed!

**Node operators DO NOT need to deploy contracts.** The smart contracts are already deployed on X1 mainnet and your node simply references them via their Program IDs in the configuration file.

### Deployed Contracts (Reference Only)

| Contract | Program ID | Description |
|----------|-----------|-------------|
| **Governance** | `TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9` | Staking, rewards, and governance |
| **L2 State Compression** | `L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx` | Merkle root storage and verification |

These Program IDs are already configured in your `node-config.toml` file. You just need to:
1. ✅ Install the node
2. ✅ Stake your TACH tokens
3. ✅ Start the service

### For Developers: Building Contracts

**Only needed if you're modifying the contracts:**

```bash
cd l2-contracts
anchor build
```

**Note:** The `l2-contracts` folder is included for reference and transparency, so you can audit the contract code. Node operators don't need to interact with it.

## 🤝 Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

**Important**: Never commit private keys or sensitive information!

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **X1 Blockchain**: https://x1.xyz
- **Explorer**: https://explorer.x1.xyz
- **GitHub**: https://github.com/xenian84/tachyon-oracles
- **Documentation**: [NEW_NODE_SETUP.md](NEW_NODE_SETUP.md)

## 💬 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/xenian84/tachyon-oracles/issues)
- **Documentation**: Check the docs folder for detailed guides

## 🎯 Roadmap

- [x] Rust node implementation
- [x] Stake-weighted consensus
- [x] L2 state compression
- [x] Governance system
- [x] Rewards distribution
- [x] Performance tracking
- [ ] Web dashboard
- [ ] Multi-chain support
- [ ] Advanced analytics
- [ ] Historical data API

## ⚡ Performance

- **Batch Submission**: Every 60 seconds
- **Price Updates**: Real-time from multiple exchanges
- **Consensus**: Stake-weighted leader selection
- **Throughput**: Handles 100+ price feeds efficiently

## 🌟 Acknowledgments

Built with ❤️ for the X1 ecosystem using:
- [Rust](https://www.rust-lang.org/)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Solana](https://solana.com/)

---

**Ready to run your own oracle node? Get started with the [Quick Start](#-quick-start) guide!**

