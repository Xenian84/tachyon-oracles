# Tachyon Oracles

A decentralized oracle network for the X1 blockchain, providing real-time price feeds for crypto assets.

## 🌟 Features

- **9 Price Feeds**: BTC/USD, ETH/USD, SOL/USD, AVAX/USD, MATIC/USD, BNB/USD, XRP/USD, ADA/USD, DOT/USD
- **Decentralized**: Multi-publisher architecture with quorum-based consensus
- **Real-time Updates**: Prices updated every ~5 seconds
- **Telegram Bot**: Monitor and manage your oracle node remotely
- **API Service**: RESTful API for monitoring and integration
- **Auto-Recovery**: Built-in health checks and automatic restarts

## 📋 Prerequisites

- Ubuntu 20.04+ or Debian 11+
- 2GB+ RAM
- 20GB+ disk space
- Node.js 18+ (will be installed automatically)
- Git

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/Xenian84/tachyon-oracles.git
cd tachyon-oracles
```

### 2. Run the Setup Wizard

```bash
./tachyon
```

Select: **8. 🚀 First Time Setup Wizard**

The wizard will automatically:
- ✅ Install all dependencies (Node.js, PM2, Solana CLI)
- ✅ Generate unique keypairs for your node
- ✅ Configure environment variables
- ✅ Build all services
- ✅ Register your node as a publisher
- ✅ Start services with PM2
- ✅ Setup API service for monitoring

### 3. Verify Services

```bash
pm2 list
```

You should see:
- `tachyon-signer` - Running
- `tachyon-relayer` - Running
- `tachyon-api` - Running

### 4. Monitor Your Node

**Option A: Console (Local)**
```bash
./tachyon
```

**Option B: Telegram Bot (Remote)**

1. Open [@tachyon_oracle_bot](https://t.me/tachyon_oracle_bot)
2. Send `/start`
3. Tap: 🗂️ Profiles → ➕ Add Profile
4. Enter your server details (API URL and key from setup)
5. Monitor status, feeds, and logs remotely!

## 📊 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Tachyon Oracle Node                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐      ┌──────────────┐      ┌──────────┐  │
│  │   Signer    │─────▶│   Relayer    │─────▶│    X1    │  │
│  │             │      │              │      │Blockchain│  │
│  │ Fetches     │      │ Aggregates   │      │          │  │
│  │ prices from │      │ signatures   │      │ On-chain │  │
│  │ exchanges   │      │ & submits    │      │  feeds   │  │
│  └─────────────┘      └──────────────┘      └──────────┘  │
│         │                     │                            │
│         └─────────────────────┴──────────────┐             │
│                                               │             │
│                                    ┌──────────▼─────────┐   │
│                                    │   API Service      │   │
│                                    │                    │   │
│                                    │  Monitoring &      │   │
│                                    │  Management        │   │
│                                    └────────────────────┘   │
│                                               │             │
└───────────────────────────────────────────────┼─────────────┘
                                                │
                                    ┌───────────▼──────────┐
                                    │   Telegram Bot       │
                                    │   (Remote Access)    │
                                    └──────────────────────┘
```

## 🔧 Configuration

### Environment Variables

The setup wizard creates a `.env` file with these key variables:

```bash
# Network
RPC_URL=https://rpc.mainnet.x1.xyz
WS_URL=wss://rpc.mainnet.x1.xyz
PROGRAM_ID=TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1

# Services
RELAYER_PORT=7777
API_PORT=7171

# Keypairs (generated automatically)
SIGNER_KEYPAIR=./keys/signer.json
RELAYER_KEYPAIR=./keys/relayer.json
PUBLISHER_KEYPAIR=./keys/publisher.json
```

### Asset Configuration

Edit `signer/config.yaml` to customize price feeds:

```yaml
assets:
  - id: "BTC/USD"
    symbol:
      binance: "BTCUSDT"
      coinbase: "BTC-USD"
      kraken: "XBTUSD"
    sources: ["binance", "coinbase", "kraken"]
```

## 📡 API Endpoints

The API service runs on port 7171:

### GET /api/status
Get service and network status
```json
{
  "services": {
    "signer": "running",
    "relayer": "running"
  },
  "network": {
    "feeds": 9,
    "publishers": 3
  }
}
```

### GET /api/feeds
Get all price feeds
```json
{
  "feeds": [
    {
      "pair": "BTC/USD",
      "price": "8729950000000",
      "confidence": "350500000",
      "timestamp": "1703620800"
    }
  ]
}
```

### GET /api/publishers
Get all registered publishers

### GET /api/logs/:service
Get service logs (signer or relayer)

## 🤖 Telegram Bot Commands

### Profile Management
- `/start` - Start the bot
- `/profiles` - List all validator profiles
- `/add_profile` - Add a new validator
- `/switch <n>` - Switch to profile #n
- `/rename_profile <name>` - Rename current profile
- `/delete_profile <n>` - Delete profile #n

### Monitoring
- `/status` - Service and network status
- `/feeds` - View all price feeds
- `/publishers` - List all publishers

### Interactive Menu
Tap buttons for quick access:
- 📊 Status
- 🗂️ Profiles
- 📈 Feeds
- 👥 Publishers
- 📝 Logs
- ❓ Help

## 🔐 Security

### Private Keys
- **NEVER** commit private keys to git
- Keys are stored in `keys/` directory (gitignored)
- Each node generates unique keypairs
- Keep your `keys/` directory backed up securely

### API Security
- API key authentication required
- IP whitelisting supported
- Rate limiting enabled
- CORS configured

### Firewall Rules
```bash
# Allow API access (if using Telegram bot)
ufw allow 7171/tcp

# Allow relayer communication (between nodes)
ufw allow 7777/tcp

# SSH (keep this open!)
ufw allow 22/tcp

ufw enable
```

## 📚 Documentation

- [Console Guide](CONSOLE_README.md) - Using the management console
- [Telegram Bot Guide](TELEGRAM_BOT_GUIDE.md) - Setting up remote monitoring
- [Multi-Profile Guide](MULTI_PROFILE_GUIDE.md) - Managing multiple validators
- [Quick Start](QUICK_START.md) - Fast deployment guide
- [Validator Setup](VALIDATOR_SETUP.md) - Detailed setup instructions

## 🛠️ Troubleshooting

### Services Not Starting

```bash
# Check logs
pm2 logs tachyon-signer
pm2 logs tachyon-relayer

# Restart services
pm2 restart all

# Check console
./tachyon
# Select: 1. Service Manager
```

### Insufficient Publishers Error

This means the network needs more publishers. The oracle requires a minimum of 3 publishers for price submissions.

**Solution**: Deploy additional nodes on other servers following the Quick Start guide.

### Port Already in Use

```bash
# Check what's using the port
sudo lsof -i :7777
sudo lsof -i :7171

# Kill the process or change the port in .env
```

### API Connection Failed

```bash
# Check if API service is running
pm2 list

# Check API logs
pm2 logs tachyon-api

# Verify firewall allows port 7171
sudo ufw status
```

## 🤝 Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details

## 🔗 Links

- **X1 Blockchain**: [https://x1.xyz](https://x1.xyz)
- **Telegram Bot**: [@tachyon_oracle_bot](https://t.me/tachyon_oracle_bot)
- **GitHub**: [https://github.com/Xenian84/tachyon-oracles](https://github.com/Xenian84/tachyon-oracles)

## 💬 Support

- **Telegram**: Join our community
- **Issues**: [GitHub Issues](https://github.com/Xenian84/tachyon-oracles/issues)
- **Documentation**: Check the `docs/` directory

## 🎯 Roadmap

- [x] Multi-publisher support
- [x] Telegram bot monitoring
- [x] API service
- [x] Auto-recovery
- [ ] Web dashboard
- [ ] More price feeds
- [ ] Historical data API
- [ ] Advanced analytics

---

**Built with ❤️ for the X1 ecosystem**
