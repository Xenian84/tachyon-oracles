# Telegram Bot Guide

Monitor and manage your Tachyon Oracle nodes remotely using the Telegram bot.

## 🤖 Bot Information

- **Bot Name**: @tachyon_oracle_bot
- **Features**: Multi-profile support, real-time monitoring, interactive menu
- **Access**: API-only (secure remote access)

## 🚀 Quick Start

### 1. Open the Bot

Search for `@tachyon_oracle_bot` in Telegram or click: https://t.me/tachyon_oracle_bot

### 2. Start the Bot

Send `/start` to begin

### 3. Add Your First Profile

1. Tap: **🗂️ Profiles**
2. Tap: **➕ Add Profile** or **🔧 Setup First Profile**
3. Enter profile name (e.g., "Main Validator")
4. Enter API URL: `http://YOUR_SERVER_IP:7171`
5. Enter API Key (from your server's setup)

### 4. Done!

You can now monitor your oracle node remotely!

## 📱 Main Menu

```
┌────────────────────────────────┐
│ 📊 Status    │ 🗂️ Profiles     │
│ 📈 Feeds     │ 👥 Publishers   │
│ 📝 Logs      │ ❓ Help         │
└────────────────────────────────┘
```

## 📊 Status

View current status of your oracle node.

**Shows:**
- Service status (signer/relayer running/stopped)
- Network info (feeds and publishers count)
- RPC connection
- Current profile name

**Usage:**
- Tap: **📊 Status**
- Or send: `/status`

**Example Output:**
```
📊 Tachyon Oracle Status
🏷️ Profile: Main Validator

Services:
Signer: ✅ Running
Relayer: ✅ Running

Network:
Feeds: 9
Publishers: 3

RPC: https://rpc.mainnet.x1.xyz
```

## 📈 Feeds

View all price feeds with current prices.

**Shows:**
- Trading pairs (BTC/USD, ETH/USD, etc.)
- Current prices with thousand separators
- Confidence intervals
- Last update timestamps
- Staleness warnings

**Usage:**
- Tap: **📈 Feeds**
- Or send: `/feeds`

**Example Output:**
```
📈 Price Feeds:

*BTC/USD*
  💰 $87,299.50 ±$3.5050
  🕐 12/26/2025, 8:45:30 PM

*ETH/USD*
  💰 $2,919.96 ±$0.0675
  🕐 12/26/2025, 8:45:30 PM

*SOL/USD*
  💰 $121.81 ±$0.0025
  🕐 12/26/2025, 8:45:30 PM
```

## 👥 Publishers

List all registered publishers on the network.

**Shows:**
- Publisher public keys
- Active/inactive status
- Total count

**Usage:**
- Tap: **👥 Publishers**
- Or send: `/publishers`

**Example Output:**
```
👥 Publishers (3):

1. ✅ Bqc3QJsDpXx91Yqn3HjPeMdDANbg5u4aPASGKcHik5h8
2. ✅ 7xKz9pRqYnM3vLbP2wQ8jNfT4sR6hC5dE1aF9bG3kH2m
3. ✅ 3mN5pQ8rT2wV9xA4bC7dE1fG6hJ9kL2nM5oP8qR3sT6u
```

## 📝 Logs

View service logs for debugging and monitoring.

**Shows:**
- Last 50 lines of service logs
- Signer or relayer logs
- Formatted in code blocks

**Usage:**
1. Tap: **📝 Logs**
2. Choose: **📝 Signer Logs** or **📝 Relayer Logs**

**Example Output:**
```
📝 SIGNER Logs (last 50 lines):

```
2025-12-26T20:45:30: BTC/USD: price=87299.50, conf=3.51, sources=2
2025-12-26T20:45:30: Submitted BTC/USD to http://localhost:7777
2025-12-26T20:45:35: ETH/USD: price=2919.96, conf=0.07, sources=2
```
```

## 🗂️ Profile Management

Manage multiple validator nodes from one bot.

### List Profiles

**Command:** `/profiles` or tap **📋 List Profiles**

Shows all your configured validators with:
- Profile number
- Name
- API URL
- Active indicator (🟢)

### Add Profile

**Command:** `/add_profile` or tap **➕ Add Profile**

Steps:
1. Enter profile name
2. Enter API URL
3. Enter API key
4. Bot tests connection
5. Profile saved!

### Switch Profile

**Command:** `/switch <number>`

Example: `/switch 2` - Switch to profile #2

### Rename Profile

**Command:** `/rename_profile <new name>`

Example: `/rename_profile Backup Server`

### Delete Profile

**Command:** `/delete_profile <number>`

Example: `/delete_profile 2` - Delete profile #2

## 🔧 Setup Instructions

### Getting Your API Key

On your server:

```bash
# View API key
cat api-service/.env | grep API_KEY
```

Or from console:
```bash
./tachyon
# Select: 1. Service Manager → 10. API Service Manager → Show API Key
```

### Getting Your API URL

Format: `http://YOUR_SERVER_IP:7171`

Example: `http://209.159.154.102:7171`

### Testing Connection

After adding a profile, the bot automatically tests the connection and shows:
- ✅ Success - Connection working
- ❌ Failed - Check URL, key, firewall

## 📋 All Commands

### Profile Management
- `/start` - Start the bot
- `/profiles` - List all profiles
- `/add_profile` - Add new validator
- `/switch <n>` - Switch to profile #n
- `/rename_profile <name>` - Rename current profile
- `/delete_profile <n>` - Delete profile #n
- `/config` - View current configuration

### Monitoring
- `/status` - Service and network status
- `/feeds` - View all price feeds
- `/publishers` - List all publishers

### Help
- `/help` - Show help message

## 💡 Tips

### Use Interactive Menu

Instead of typing commands, just tap the buttons! The menu is always visible and makes navigation easier.

### Monitor Multiple Validators

You can add multiple profiles and switch between them:

```
/switch 1  → Check Main Validator
/switch 2  → Check Backup Server
/switch 3  → Check Test Node
```

### Check Regularly

- Check `/status` daily to ensure services are running
- Check `/feeds` to verify prices are updating
- Check `/publishers` to see network health

## 🔒 Security

### API Key Protection

- Never share your API key publicly
- Each server has a unique API key
- Keys are stored encrypted in the bot

### IP Whitelisting

On your server, restrict API access:

```bash
# Edit api-service/.env
ALLOWED_IPS=YOUR_IP,TELEGRAM_BOT_IP
```

### Firewall

Ensure port 7171 is only accessible from trusted IPs:

```bash
sudo ufw allow from YOUR_IP to any port 7171
```

## ❌ Troubleshooting

### "API Connection Failed"

**Causes:**
- API service not running
- Wrong URL or API key
- Firewall blocking port 7171
- Server is down

**Solutions:**
1. Check API service: `pm2 list`
2. Verify API key: `cat api-service/.env | grep API_KEY`
3. Check firewall: `sudo ufw status`
4. Test locally: `curl http://localhost:7171/api/status`

### "No profiles configured"

**Solution:** Add a profile using `/add_profile` or tap **➕ Add Profile**

### Feeds Showing $0.00

**Cause:** Network has fewer than 3 publishers

**Solution:** Deploy more publisher nodes or wait for others to join

### Bot Not Responding

**Solutions:**
1. Send `/start` to restart
2. Check if bot is online: https://t.me/tachyon_oracle_bot
3. Try again in a few minutes

## 🎯 Example Workflow

### Daily Monitoring

```
1. Open bot
2. Tap: 📊 Status
   → Check services are running
3. Tap: 📈 Feeds
   → Verify prices are updating
4. Done! ✅
```

### Adding Second Validator

```
1. Deploy new server following DEPLOYMENT.md
2. Get API URL and key from new server
3. In bot: 🗂️ Profiles → ➕ Add Profile
4. Enter details for new server
5. Switch between servers with /switch
```

### Debugging Issues

```
1. Tap: 📊 Status
   → Check which service is down
2. Tap: 📝 Logs
   → View logs for that service
3. Look for errors
4. Fix on server
5. Verify with 📊 Status
```

## 📞 Support

- **GitHub**: [Open an issue](https://github.com/Xenian84/tachyon-oracles/issues)
- **Telegram**: Join the community
- **Documentation**: Check [README.md](README.md) and [CONSOLE_GUIDE.md](CONSOLE_GUIDE.md)

---

**Happy monitoring!** 🚀
