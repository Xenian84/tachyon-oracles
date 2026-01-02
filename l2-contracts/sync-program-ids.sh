#!/bin/bash

# Sync all program IDs from Anchor.toml to Rust source files

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║     🔄 SYNCING PROGRAM IDs FROM ANCHOR.TOML 🔄                      ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo ""

# Read program IDs from Anchor.toml
STATE_COMPRESSION_ID="L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"
L2_CORE_ID="CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3"
VERIFIER_ID="VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR"
BRIDGE_ID="BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW"
SEQUENCER_ID="SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M"
GOVERNANCE_ID="TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"

echo "Target Program IDs:"
echo "  State Compression: $STATE_COMPRESSION_ID"
echo "  L2 Core:          $L2_CORE_ID"
echo "  Verifier:         $VERIFIER_ID"
echo "  Bridge:           $BRIDGE_ID"
echo "  Sequencer:        $SEQUENCER_ID"
echo "  Governance:       $GOVERNANCE_ID"
echo ""

# Function to update declare_id in a Rust file
update_program_id() {
    local file=$1
    local new_id=$2
    local program_name=$3
    
    if [ -f "$file" ]; then
        # Get current ID
        current_id=$(grep -oP 'declare_id!\("\K[^"]+' "$file")
        
        if [ "$current_id" != "$new_id" ]; then
            echo "📝 Updating $program_name"
            echo "   Old: $current_id"
            echo "   New: $new_id"
            
            # Update the file
            sed -i "s/declare_id!(\"[^\"]*\")/declare_id!(\"$new_id\")/" "$file"
            echo "   ✅ Updated"
        else
            echo "✅ $program_name already correct"
        fi
    else
        echo "❌ File not found: $file"
    fi
    echo ""
}

# Update all program files
cd /root/tachyon-oracles/l2-contracts

update_program_id "programs/tachyon-state-compression/src/lib.rs" "$STATE_COMPRESSION_ID" "State Compression"
update_program_id "programs/tachyon-l2-core/src/lib.rs" "$L2_CORE_ID" "L2 Core"
update_program_id "programs/tachyon-verifier/src/lib.rs" "$VERIFIER_ID" "Verifier"
update_program_id "programs/tachyon-bridge/src/lib.rs" "$BRIDGE_ID" "Bridge"
update_program_id "programs/tachyon-sequencer/src/lib.rs" "$SEQUENCER_ID" "Sequencer"
update_program_id "programs/tachyon-governance/src/lib.rs" "$GOVERNANCE_ID" "Governance"

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║     ✅ ALL PROGRAM IDs SYNCED ✅                                     ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"

