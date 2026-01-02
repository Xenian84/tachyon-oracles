# 🚀 Tachyon Oracle - New Node Setup Guide

Complete guide to set up a new Tachyon Oracle node from scratch.

## Prerequisites

- Ubuntu/Debian Linux server
- Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Solana CLI installed (`sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`)
- Node.js & npm installed
- At least 100,000 TACH tokens for staking (minimum requirement)

---

## Step 1: Create New Keypair

```bash
# Create a new keypair for your node
solana-keygen new -o ~/new-node-keypair.json

# Get the public key
solana-keygen pubkey ~/new-node-keypair.json

# Check balance (you'll need some XNT for transaction fees)
solana balance ~/new-node-keypair.json --url https://rpc.mainnet.x1.xyz
```

**⚠️ IMPORTANT:** Save your keypair securely! Back it up!

---

## Step 2: Fund the Node Wallet

```bash
# You need:
# 1. ~0.1 XNT for transaction fees
# 2. 100,000+ TACH tokens for staking (minimum)

# Transfer XNT for fees (from your main wallet)
solana transfer \
  --url https://rpc.mainnet.x1.xyz \
  --keypair ~/.config/solana/id.json \
  <NEW_NODE_PUBKEY> \
  0.1

# Transfer TACH tokens (minimum 100,000, recommended 200,000+)
spl-token transfer \
  --url https://rpc.mainnet.x1.xyz \
  --owner ~/.config/solana/id.json \
  TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu \
  100000 \
  <NEW_NODE_PUBKEY>
```

---

## Step 3: Clone and Build Tachyon Node

```bash
# Clone the repository (or copy from existing installation)
cd /root
git clone <your-tachyon-repo> tachyon-node-2
cd tachyon-node-2/tachyon-node

# Build the node
cargo build --release

# Install the binary
sudo cp target/release/tachyon-node /usr/local/bin/tachyon-node-2
```

---

## Step 4: Create Node Configuration

```bash
# Create config directory
sudo mkdir -p /etc/tachyon-node-2

# Create the configuration file
sudo nano /etc/tachyon-node-2/node-config.toml
```

**Configuration template:**

```toml
keypair_path = "/var/lib/tachyon-node-2/node-keypair.json"
rpc_url = "https://rpc.mainnet.x1.xyz"
program_id = "TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"
l2_program_id = "L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"
gossip_port = 9001
api_port = 7778
update_interval_ms = 1000
batch_interval_ms = 100
min_publishers = 3

[[assets]]
symbol = "BTC/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "ETH/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "SOL/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "AVAX/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "MATIC/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "BNB/USD"
exchanges = ["binance"]

[[assets]]
symbol = "XRP/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "ADA/USD"
exchanges = ["binance", "coinbase"]

[[assets]]
symbol = "DOT/USD"
exchanges = ["binance", "coinbase"]

[exchanges]
```

---

## Step 5: Set Up Keypair

```bash
# Create keypair directory
sudo mkdir -p /var/lib/tachyon-node-2

# Copy your new keypair
sudo cp ~/new-node-keypair.json /var/lib/tachyon-node-2/node-keypair.json

# Set proper permissions
sudo chown root:root /var/lib/tachyon-node-2/node-keypair.json
sudo chmod 600 /var/lib/tachyon-node-2/node-keypair.json
```

---

## Step 6: Stake TACH Tokens

```bash
# Initialize staker account and stake (minimum 100,000 TACH)
tachyon-node-2 stake \
  --config /etc/tachyon-node-2/node-config.toml \
  --amount 100000

# Verify stake
tachyon-node-2 view-stake-info \
  --config /etc/tachyon-node-2/node-config.toml
```

**Expected output:**
```
╔══════════════════════════════════════════════════════════════╗
║              📊 DETAILED STAKE INFORMATION                   ║
╠══════════════════════════════════════════════════════════════╣
║ 💰 Staked Amount:        100000000.00 TACH                   ║
║ 📅 Staked Since:         2026-01-02 XX:XX                    ║
...
```

---

## Step 7: Create Systemd Service

```bash
sudo nano /etc/systemd/system/tachyon-node-2.service
```

**Service file:**

```ini
[Unit]
Description=Tachyon Oracle Node 2
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/tachyon-node-2/tachyon-node
ExecStart=/usr/local/bin/tachyon-node-2 run --config /etc/tachyon-node-2/node-config.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=tachyon-node-2

[Install]
WantedBy=multi-user.target
```

**Enable and start the service:**

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable tachyon-node-2

# Start the service
sudo systemctl start tachyon-node-2

# Check status
sudo systemctl status tachyon-node-2

# View logs
sudo journalctl -u tachyon-node-2 -f
```

---

## Step 8: Verify Node is Working

### Check if node is running:
```bash
sudo systemctl status tachyon-node-2
```

### Check logs for batch submissions:
```bash
sudo journalctl -u tachyon-node-2 -f | grep -i "submit"
```

### View stake information:
```bash
tachyon-node-2 view-stake-info --config /etc/tachyon-node-2/node-config.toml
```

### View performance metrics:
```bash
tachyon-node-2 view-performance --config /etc/tachyon-node-2/node-config.toml
```

### Check L2 state:
```bash
node << 'EOF'
const { Connection, PublicKey } = require('@solana/web3.js');

const connection = new Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
const STATE_COMPRESSION_PROGRAM_ID = new PublicKey('L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx');

(async () => {
    const [l2StatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('l2-state')],
        STATE_COMPRESSION_PROGRAM_ID
    );
    
    const accountInfo = await connection.getAccountInfo(l2StatePDA);
    
    if (accountInfo) {
        let offset = 8;
        offset += 32; // authority
        offset += 32; // current_root
        
        const batchNumber = accountInfo.data.readBigUInt64LE(offset);
        
        console.log('Current Batch Number:', batchNumber.toString());
        console.log('✅ Your node should be incrementing this number!');
    }
})();
EOF
```

---

## Step 9: Monitor and Verify

### Things to check:

1. **Node is running:**
   ```bash
   sudo systemctl status tachyon-node-2
   ```

2. **Logs show activity:**
   ```bash
   sudo journalctl -u tachyon-node-2 --since "5 minutes ago"
   ```

3. **Stake is active:**
   ```bash
   tachyon-node-2 view-stake-info --config /etc/tachyon-node-2/node-config.toml
   ```

4. **Batch number is incrementing:**
   - Wait 60 seconds between checks
   - Run the L2 state check script above
   - Batch number should increase

5. **No errors in logs:**
   ```bash
   sudo journalctl -u tachyon-node-2 -p err
   ```

---

## Step 10: Set Up Firewall (Optional)

```bash
# Allow gossip port
sudo ufw allow 9001/tcp

# Allow API port (if exposing)
sudo ufw allow 7778/tcp

# Reload firewall
sudo ufw reload
```

---

## Troubleshooting

### Node won't start:
```bash
# Check logs
sudo journalctl -u tachyon-node-2 -n 50

# Check config file syntax
cat /etc/tachyon-node-2/node-config.toml

# Check keypair permissions
ls -la /var/lib/tachyon-node-2/node-keypair.json
```

### Not submitting batches:
```bash
# Check if node is leader
sudo journalctl -u tachyon-node-2 -f | grep -i "leader"

# Check stake
tachyon-node-2 view-stake-info --config /etc/tachyon-node-2/node-config.toml

# Check for errors
sudo journalctl -u tachyon-node-2 -p err
```

### Insufficient funds errors:
```bash
# Check XNT balance
solana balance /var/lib/tachyon-node-2/node-keypair.json --url https://rpc.mainnet.x1.xyz

# Add more XNT if needed
solana transfer \
  --url https://rpc.mainnet.x1.xyz \
  --keypair ~/.config/solana/id.json \
  <NEW_NODE_PUBKEY> \
  0.1
```

---

## Quick Commands Reference

```bash
# Start node
sudo systemctl start tachyon-node-2

# Stop node
sudo systemctl stop tachyon-node-2

# Restart node
sudo systemctl restart tachyon-node-2

# View logs
sudo journalctl -u tachyon-node-2 -f

# Check status
sudo systemctl status tachyon-node-2

# View stake
tachyon-node-2 view-stake-info --config /etc/tachyon-node-2/node-config.toml

# View performance
tachyon-node-2 view-performance --config /etc/tachyon-node-2/node-config.toml

# Claim rewards
tachyon-node-2 claim-rewards --config /etc/tachyon-node-2/node-config.toml
```

---

## Success Checklist

- [ ] Keypair created and backed up
- [ ] Wallet funded with XNT and TACH
- [ ] Node built and installed
- [ ] Configuration file created
- [ ] Keypair copied to /var/lib/tachyon-node-2/
- [ ] TACH staked (200,000+)
- [ ] Systemd service created and enabled
- [ ] Node started successfully
- [ ] Logs show no errors
- [ ] Stake info displays correctly
- [ ] Batch number is incrementing
- [ ] Node is submitting batches

---

## 🎉 Congratulations!

Your new Tachyon Oracle node is now operational!

Monitor it for the first few hours to ensure everything is working correctly.
Check the batch number periodically to confirm your node is participating
in consensus and submitting batches to the L2 state compression contract.

**Next steps:**
- Monitor performance metrics
- Claim rewards after 24 hours
- Join the validator community
- Keep your node updated

---

**Need help?** Check the logs first:
```bash
sudo journalctl -u tachyon-node-2 -f
```

