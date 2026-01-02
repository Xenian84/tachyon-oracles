# 🚀 L2 Contracts Setup Guide

**ALWAYS follow this sequence to avoid program ID mess!**

---

## 📋 Prerequisites

- Deployer wallet: `/root/.config/tachyon/deployer.json`
- Vanity keypairs: `/root/tachyon-oracles/keys/`
- X1 RPC: `https://rpc.mainnet.x1.xyz`

---

## 🔧 Setup Steps (Run in Order)

### Step 1: Setup Vanity Addresses
```bash
cd /root/tachyon-oracles/l2-contracts
./setup-vanity-addresses.sh
```

**What it does:**
- Copies vanity keypairs from `/root/tachyon-oracles/keys/` to `target/deploy/`
- Ensures Anchor uses the correct program IDs
- Verifies all keypairs are in place

### Step 2: Sync Program IDs
```bash
./sync-program-ids.sh
```

**What it does:**
- Reads program IDs from `Anchor.toml`
- Updates `declare_id!()` in all Rust source files
- Prevents "DeclaredProgramIdMismatch" errors

### Step 3: Build Contracts
```bash
anchor build
```

**What it does:**
- Compiles all 6 contracts
- Generates IDLs in `target/idl/`
- Creates `.so` files in `target/sbpf-solana-solana/release/`

### Step 4: Deploy/Upgrade Contracts
```bash
anchor deploy --provider.cluster https://rpc.mainnet.x1.xyz
```

**What it does:**
- Deploys to X1 mainnet
- Uses vanity addresses from `target/deploy/*.json`
- Upgrades existing programs if already deployed

---

## 🎯 Program IDs (Vanity Addresses)

| Contract | Program ID | Keypair Location |
|----------|-----------|------------------|
| **State Compression** | `L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx` | `keys/l2-program-keypair.json` |
| **Governance** | `TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9` | `keys/tachyon-governance-keypair.json` |
| **Sequencer** | `SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M` | `keys/tachyon-sequencer-keypair.json` |
| **Verifier** | `VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR` | `keys/tachyon-verifier-keypair.json` |
| **Bridge** | `BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW` | `keys/tachyon-bridge-keypair.json` |
| **L2 Core** | `CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3` | `keys/tachyon-l2-core-keypair.json` |

---

## ⚠️ IMPORTANT: Program ID Locations

### Source of Truth
**File**: `Anchor.toml` (lines 8-14)
```toml
[programs.localnet]
tachyon_state_compression = "L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"
tachyon_l2_core = "CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3"
tachyon_verifier = "VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR"
tachyon_bridge = "BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW"
tachyon_sequencer = "SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M"
tachyon_governance = "TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"
```

### Rust Files (Auto-synced)
Each program has `declare_id!()` in `programs/*/src/lib.rs`:
```rust
declare_id!("L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx");
```

**NEVER edit these manually!** Use `sync-program-ids.sh` instead.

### Keypair Files (For Deployment)
Location: `target/deploy/*.json`

These must match the program IDs in `Anchor.toml`. Use `setup-vanity-addresses.sh` to copy them.

---

## 🔄 Complete Workflow

```bash
# 1. Setup (first time or after clean)
cd /root/tachyon-oracles/l2-contracts
./setup-vanity-addresses.sh

# 2. Sync IDs (before every build)
./sync-program-ids.sh

# 3. Build
anchor build

# 4. Deploy
anchor deploy --provider.cluster https://rpc.mainnet.x1.xyz

# 5. Verify
solana program show L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx --url https://rpc.mainnet.x1.xyz
```

---

## ❌ Common Mistakes

### 1. Forgetting to run setup-vanity-addresses.sh
**Result**: Anchor generates new random program IDs

**Fix**: Always run `setup-vanity-addresses.sh` first

### 2. Forgetting to run sync-program-ids.sh
**Result**: "DeclaredProgramIdMismatch" error

**Fix**: Always run `sync-program-ids.sh` before building

### 3. Running `cargo clean` or `anchor clean`
**Result**: Deletes `target/deploy/*.json` keypairs

**Fix**: Run `setup-vanity-addresses.sh` again

### 4. Manually editing declare_id!()
**Result**: Out of sync with Anchor.toml

**Fix**: Edit `Anchor.toml`, then run `sync-program-ids.sh`

---

## 🛠️ Troubleshooting

### Problem: "DeclaredProgramIdMismatch"
```bash
./sync-program-ids.sh
anchor build
anchor deploy
```

### Problem: New random program IDs generated
```bash
./setup-vanity-addresses.sh
anchor build
anchor deploy
```

### Problem: "Program file not found"
```bash
# Check if .so files exist
ls -la target/sbpf-solana-solana/release/*.so

# If missing, rebuild
anchor build
```

### Problem: "Insufficient funds"
```bash
# Check deployer balance
solana balance /root/.config/tachyon/deployer.json --url https://rpc.mainnet.x1.xyz

# Fund if needed
# (transfer X1 tokens to deployer address)
```

---

## 📝 Quick Reference

### Check Program IDs
```bash
# From Anchor.toml
grep "tachyon_" Anchor.toml

# From Rust files
grep -r "declare_id" programs/*/src/lib.rs

# From keypairs
for f in target/deploy/*.json; do 
    echo "$(basename $f): $(solana-keygen pubkey $f)"
done
```

### Verify Deployment
```bash
# Check if program exists on-chain
solana program show <PROGRAM_ID> --url https://rpc.mainnet.x1.xyz

# Check program authority
solana program show <PROGRAM_ID> --url https://rpc.mainnet.x1.xyz | grep Authority
```

---

## ✅ Checklist

Before deploying, verify:

- [ ] Vanity keypairs in `target/deploy/`
- [ ] `Anchor.toml` has correct program IDs
- [ ] Rust files have matching `declare_id!()`
- [ ] `anchor build` completes successfully
- [ ] Deployer wallet has sufficient X1 balance
- [ ] RPC URL is correct in `Anchor.toml`

---

**Last Updated**: December 30, 2025  
**Status**: ✅ Production Ready

