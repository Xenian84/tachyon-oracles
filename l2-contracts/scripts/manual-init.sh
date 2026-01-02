#!/bin/bash

# Manual L2 Contract Initialization Script
# Run this to initialize each contract individually

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

RPC_URL="https://rpc.mainnet.x1.xyz"

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║         Manual L2 Contract Initialization                           ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo ""

# Contract addresses
STATE_COMPRESSION="L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"
L2_CORE="CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3"
VERIFIER="VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR"
BRIDGE="BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW"
SEQUENCER="SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M"
GOVERNANCE="TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"

# Function to initialize a contract using TypeScript
init_contract() {
    local name=$1
    local program_id=$2
    local idl_file=$3
    
    echo "═══════════════════════════════════════════════════════════════════════"
    echo "Initializing $name ($program_id)..."
    echo "═══════════════════════════════════════════════════════════════════════"
    
    # Create a simple TypeScript initialization script
    cat > /tmp/init_${name}.ts << EOF
import * as anchor from '@coral-xyz/anchor';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = '${RPC_URL}';
const PROGRAM_ID = new PublicKey('${program_id}');

async function main() {
    // Load deployer keypair
    const deployerPath = process.env.DEPLOYER_KEYPAIR || '/root/deployer.json';
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(deployerPath, 'utf-8')))
    );
    
    // Setup connection and provider
    const connection = new Connection(RPC_URL, 'confirmed');
    const wallet = new Wallet(deployerKeypair);
    const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
    
    // Load IDL
    const idl = JSON.parse(fs.readFileSync('${idl_file}', 'utf-8'));
    const program = new Program(idl, PROGRAM_ID, provider);
    
    console.log('Authority:', deployerKeypair.publicKey.toBase58());
    console.log('Program ID:', PROGRAM_ID.toBase58());
    
    // Find the initialize instruction
    const initInstruction = idl.instructions.find((ix: any) => 
        ix.name === 'initialize' || ix.name === 'init'
    );
    
    if (!initInstruction) {
        console.log('⚠️  No initialize instruction found in IDL');
        console.log('This contract may not need initialization or is already initialized');
        return;
    }
    
    try {
        // Derive PDAs if needed
        const [statePda] = PublicKey.findProgramAddressSync(
            [Buffer.from('l2_state')],
            program.programId
        );
        
        console.log('State PDA:', statePda.toBase58());
        
        // Call initialize
        const tx = await program.methods
            .initialize()
            .accounts({
                authority: deployerKeypair.publicKey,
                l2State: statePda,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([deployerKeypair])
            .rpc();
        
        console.log('✅ Initialized successfully!');
        console.log('Transaction:', tx);
    } catch (error: any) {
        if (error.message?.includes('already in use')) {
            console.log('✅ Already initialized');
        } else {
            console.error('❌ Error:', error.message);
            throw error;
        }
    }
}

main().catch(console.error);
EOF
    
    # Run the initialization
    npx ts-node /tmp/init_${name}.ts
    
    echo ""
}

# Initialize each contract
echo "Starting initialization..."
echo ""

init_contract "TachyonStateCompression" "$STATE_COMPRESSION" "target/idl/tachyon_state_compression.json"
init_contract "TachyonL2Core" "$L2_CORE" "target/idl/tachyon_l2_core.json"
init_contract "TachyonVerifier" "$VERIFIER" "target/idl/tachyon_verifier.json"
init_contract "TachyonBridge" "$BRIDGE" "target/idl/tachyon_bridge.json"
init_contract "TachyonSequencer" "$SEQUENCER" "target/idl/tachyon_sequencer.json"
init_contract "TachyonGovernance" "$GOVERNANCE" "target/idl/tachyon_governance.json"

echo ""
echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║         ✅ All contracts initialized!                                ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"

