# ⚡ Tachyon Oracle Node - Quick Start Guide

**Get your node running in under 10 minutes!** 🚀

---

## 🎯 One-Command Installation

```bash
curl -sSL https://raw.githubusercontent.com/xenian84/tachyon-oracles/main/install.sh | bash
```

**That's it!** The script handles everything automatically.

---

## 📋 What You Need Before Starting

### 1. **Server Requirements**
- Ubuntu 20.04+ or Debian 11+
- 2+ CPU cores (4+ recommended)
- 4+ GB RAM (8+ recommended)
- 20+ GB disk space (50+ recommended)
- Stable internet connection

### 2. **Tokens Required**
- **100,000 TACH** (minimum stake to participate)
- **0.1 XNT** (for transaction fees)

### 3. **Access**
- SSH access to your server
- Sudo privileges

---

## 🚀 Installation Steps

### Step 1: Run the Installer

SSH into your server and run:

```bash
curl -sSL https://raw.githubusercontent.com/xenian84/tachyon-oracles/main/install.sh | bash
```

The installer will:
1. ✅ Check system requirements
2. ✅ Install dependencies (Rust, Solana CLI, etc.)
3. ✅ Clone the repository
4. ✅ Build the node (~5-10 minutes)
5. ✅ Generate a keypair
6. ✅ Create configuration
7. ✅ Set up systemd service
8. ✅ Install management console

### Step 2: Save Your Node Pubkey

The installer will show you your node's public key:

```
✅ Keypair generated
ℹ️  Node pubkey: GyTVY3PjD4xxVfVVLC9g7Umyb1gmXDexQwTtb5VAELon

⚠️  IMPORTANT: Save this pubkey! You'll need it for staking.
```

**Copy this pubkey somewhere safe!** You'll need it for staking.

### Step 3: Fund Your Node

Your node needs XNT for transaction fees:

```bash
# Send 0.1 XNT to your node pubkey
solana transfer <YOUR_NODE_PUBKEY> 0.1 --url https://rpc.mainnet.x1.xyz
```

### Step 4: Stake TACH Tokens

Your node needs to stake TACH to participate:

```bash
# Option 1: Use the console
tachyon-console
# Select [3] Stake Management

# Option 2: Use CLI
tachyon-node stake --amount 100000 --config /etc/tachyon/node-config.toml
```

### Step 5: Verify Node is Running

```bash
# Check service status
systemctl status tachyon-node

# View live logs
journalctl -u tachyon-node -f

# Check stake
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml
```

You should see logs like:

```
✅ Found our stake: 100000 TACH
👑 We are the leader for slot 20132312
🚀 Submitting Merkle root to X1: b4eb391b
✅ Merkle root submitted! Tx: 46ZNuNv...
📊 Submitting price feeds...
✅ Submitted 3 price feeds
```

**Congratulations! Your node is now running!** 🎉

---

## 🎮 Managing Your Node

### Using the Console (Recommended)

```bash
tachyon-console
```

This gives you a user-friendly menu:

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
```

### Using systemctl

```bash
# Start node
sudo systemctl start tachyon-node

# Stop node
sudo systemctl stop tachyon-node

# Restart node
sudo systemctl restart tachyon-node

# Check status
systemctl status tachyon-node

# View logs
journalctl -u tachyon-node -f
```

### Using CLI

```bash
# View stake info
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml

# View performance
tachyon-node view-performance --config /etc/tachyon/node-config.toml

# View rewards
tachyon-node view-rewards --config /etc/tachyon/node-config.toml

# Claim rewards
tachyon-node claim-rewards --config /etc/tachyon/node-config.toml
```

---

## 📊 What to Monitor

### 1. **Node Status**
```bash
systemctl status tachyon-node
```
Should show: `Active: active (running)`

### 2. **Logs**
```bash
journalctl -u tachyon-node -f
```
Look for:
- ✅ `Found our stake: X TACH`
- ✅ `We are the leader for slot X`
- ✅ `Merkle root submitted!`
- ✅ `Submitted 3 price feeds`

### 3. **Stake**
```bash
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml
```
Should show your staked amount and status.

### 4. **Performance**
```bash
tachyon-node view-performance --config /etc/tachyon/node-config.toml
```
Shows your uptime score and submission accuracy.

### 5. **Wallet Balance**
```bash
solana balance /var/lib/tachyon/node-keypair.json --url https://rpc.mainnet.x1.xyz
```
Should have at least 0.01 XNT for fees.

---

## 🔧 Troubleshooting

### Node Won't Start

**Check logs:**
```bash
journalctl -u tachyon-node -n 50
```

**Common issues:**
- **Insufficient funds:** Send more XNT to node wallet
- **Not staked:** Stake at least 100,000 TACH
- **Config error:** Check `/etc/tachyon/node-config.toml`

### Node Not Submitting

**Check if you're staked:**
```bash
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml
```

**Check wallet balance:**
```bash
solana balance /var/lib/tachyon/node-keypair.json --url https://rpc.mainnet.x1.xyz
```

**Restart node:**
```bash
sudo systemctl restart tachyon-node
```

### Low Performance Score

**Check logs for errors:**
```bash
journalctl -u tachyon-node -f | grep -i error
```

**Ensure stable internet connection**

**Keep node running 24/7**

---

## 💰 Earning Rewards

Your node earns rewards for:
1. **Uptime** - Keep your node running
2. **Accuracy** - Submit accurate prices
3. **Stake** - Higher stake = higher rewards

### Check Rewards

```bash
tachyon-node view-rewards --config /etc/tachyon/node-config.toml
```

### Claim Rewards

```bash
tachyon-node claim-rewards --config /etc/tachyon/node-config.toml
```

---

## 🔐 Security Tips

1. **Secure your keypair** - Never share `/var/lib/tachyon/node-keypair.json`
2. **Use firewall** - Only expose necessary ports
3. **Keep updated** - Regularly update your node
4. **Monitor logs** - Watch for suspicious activity
5. **Backup config** - Keep backups of your configuration

### Recommended Firewall Setup

```bash
# Allow SSH
sudo ufw allow 22/tcp

# Allow API (if you want to expose it)
sudo ufw allow 8080/tcp

# Enable firewall
sudo ufw enable
```

---

## 📈 Next Steps

Once your node is running:

1. **Join Discord** - Connect with other validators *(coming soon)*
2. **Monitor Dashboard** - Track your performance *(coming soon)*
3. **Optimize Setup** - Improve your server specs
4. **Increase Stake** - Higher stake = higher rewards
5. **Help Others** - Share your experience

---

## 🆘 Getting Help

### Check Documentation
- Full README: [README.md](README.md)
- Detailed Setup: [NEW_NODE_SETUP.md](NEW_NODE_SETUP.md)
- Price Feeds: [PRICE_FEEDS_CONTRACT.md](PRICE_FEEDS_CONTRACT.md)

### Community Support
- Discord: *(coming soon)*
- GitHub Issues: [Report a bug](https://github.com/xenian84/tachyon-oracles/issues)
- Email: support@tachyon.xyz *(coming soon)*

### Common Commands Reference

```bash
# Node management
tachyon-console                    # Open management console
systemctl status tachyon-node      # Check status
journalctl -u tachyon-node -f      # View logs

# Stake management
tachyon-node view-stake-info --config /etc/tachyon/node-config.toml
tachyon-node stake --amount 100000 --config /etc/tachyon/node-config.toml
tachyon-node unstake --amount 50000 --config /etc/tachyon/node-config.toml

# Performance & rewards
tachyon-node view-performance --config /etc/tachyon/node-config.toml
tachyon-node view-rewards --config /etc/tachyon/node-config.toml
tachyon-node claim-rewards --config /etc/tachyon/node-config.toml

# Wallet
solana balance /var/lib/tachyon/node-keypair.json --url https://rpc.mainnet.x1.xyz
```

---

## ✅ Checklist

Before going live, make sure:

- [ ] Server meets minimum requirements
- [ ] Node is installed and running
- [ ] Keypair is generated and backed up
- [ ] Node wallet has 0.1+ XNT for fees
- [ ] Staked at least 100,000 TACH
- [ ] Logs show successful submissions
- [ ] Firewall is configured
- [ ] Monitoring is set up

---

## 🎉 You're All Set!

Your Tachyon Oracle node is now contributing to the decentralized price feed network!

**Welcome to the Tachyon community!** 🚀

---

*Questions? Join our Discord or open a GitHub issue!*

