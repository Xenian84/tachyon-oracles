# Tachyon Unified Node (Rust)

## 🚧 Status: Work in Progress

This is the next-generation unified node implementation in Rust. It replaces all TypeScript services with a single binary.

### ✅ What's Complete

- **All core modules implemented:**
  - `src/main.rs` - CLI interface
  - `src/config/` - Configuration management
  - `src/crypto/` - Keypair & signing
  - `src/fetcher/` - Multi-exchange price fetching
  - `src/aggregator/` - Merkle tree builder
  - `src/gossip/` - P2P network (libp2p)
  - `src/consensus/` - Stake-weighted voting
  - `src/sequencer/` - On-chain submission
  - `src/api/` - REST API & metrics
  - `src/metrics/` - Prometheus integration

- **Infrastructure:**
  - `tachyon-node.service` - systemd service
  - `install.sh` - Installation script
  - `UNIFIED_NODE_GUIDE.md` - User documentation

### ⚠️ Current Issue

**Dependency conflicts** between:
- `solana-sdk` (1.18 vs 2.1)
- `anchor-client` (0.29 vs 0.30)
- `libp2p` (0.52 vs 0.53)
- `ed25519-dalek` (version mismatches)

### 🔧 How to Fix

1. **Test version combinations:**
   ```bash
   # Try different Solana versions
   cargo tree | grep solana-sdk
   cargo tree | grep libp2p
   ```

2. **Use cargo-edit to update:**
   ```bash
   cargo install cargo-edit
   cargo upgrade
   ```

3. **Or simplify P2P:**
   - Remove libp2p dependency
   - Implement simple TCP gossip
   - Less complex, still works

### 📚 Documentation

See `UNIFIED_NODE_GUIDE.md` for complete usage instructions (once built).

### 🚀 When Ready

Once dependencies are fixed:

```bash
# Build
cargo build --release

# Install
sudo ./install.sh

# Use
tachyon-node init
tachyon-node start
```

### 💡 Note

The TypeScript system is production-ready and can be used immediately. This Rust node is an architectural improvement that can be deployed later.

---

**For now, use the TypeScript services. This Rust node will be ready in 2-3 weeks.**

