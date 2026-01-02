#!/bin/bash

# Setup Vanity Addresses - Copy vanity keypairs to correct locations
# This ensures Anchor uses the correct program IDs during deployment

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║     🔑 SETTING UP VANITY PROGRAM ADDRESSES 🔑                       ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo ""

KEYS_DIR="/root/tachyon-oracles/keys"
DEPLOY_DIR="/root/tachyon-oracles/l2-contracts/target/deploy"

# Create deploy directory if it doesn't exist
mkdir -p "$DEPLOY_DIR"

echo "Copying vanity keypairs to target/deploy/..."
echo ""

# Map of vanity keypairs to their target names
declare -A KEYPAIRS=(
    ["tachyon-bridge-keypair.json"]="tachyon_bridge-keypair.json"
    ["tachyon-governance-keypair.json"]="tachyon_governance-keypair.json"
    ["tachyon-l2-core-keypair.json"]="tachyon_l2_core-keypair.json"
    ["tachyon-sequencer-keypair.json"]="tachyon_sequencer-keypair.json"
    ["tachyon-verifier-keypair.json"]="tachyon_verifier-keypair.json"
)

# L2 State Compression needs special handling (we need to find/generate it)
# For now, we'll handle it separately

for source in "${!KEYPAIRS[@]}"; do
    target="${KEYPAIRS[$source]}"
    source_path="$KEYS_DIR/$source"
    target_path="$DEPLOY_DIR/$target"
    
    if [ -f "$source_path" ]; then
        cp "$source_path" "$target_path"
        pubkey=$(solana-keygen pubkey "$target_path" 2>/dev/null)
        echo "✅ $target"
        echo "   Address: $pubkey"
    else
        echo "❌ $source not found in $KEYS_DIR"
    fi
done

echo ""

# Handle L2 State Compression (L2TA...)
echo "Checking L2 State Compression keypair..."
L2_STATE_KEYPAIR="$DEPLOY_DIR/tachyon_state_compression-keypair.json"

if [ -f "$L2_STATE_KEYPAIR" ]; then
    pubkey=$(solana-keygen pubkey "$L2_STATE_KEYPAIR" 2>/dev/null)
    if [[ "$pubkey" == L2TA* ]]; then
        echo "✅ tachyon_state_compression-keypair.json"
        echo "   Address: $pubkey"
    else
        echo "⚠️  Existing keypair doesn't match L2TA prefix: $pubkey"
        echo "   Keeping existing keypair (already deployed)"
    fi
else
    echo "❌ L2 State Compression keypair not found"
    echo "   Need to generate or locate L2TA... keypair"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║     ✅ VANITY ADDRESSES SETUP COMPLETE ✅                            ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Now run: ./sync-program-ids.sh"
echo "Then:    anchor build"
echo "Then:    anchor deploy"

