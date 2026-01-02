import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import * as fs from 'fs';

const RPC_URL = 'https://rpc.mainnet.x1.xyz';
const connection = new Connection(RPC_URL, 'confirmed');

// Load deployer keypair
const deployerKeypairPath = process.env.DEPLOYER_KEYPAIR || '/root/.config/tachyon/deployer.json';
const deployerKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(deployerKeypairPath, 'utf-8')))
);

const provider = new AnchorProvider(connection, new Wallet(deployerKeypair), { commitment: 'confirmed' });

const TACH_MINT = new PublicKey("TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu");

// Contract addresses
const CONTRACTS = {
    stateCompression: new PublicKey("L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"),
    l2Core: new PublicKey("CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3"),
    verifier: new PublicKey("VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR"),
    bridge: new PublicKey("BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW"),
    sequencer: new PublicKey("SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M"),
    governance: new PublicKey("TACHaFSmJJ1i6UXR1KkjxSP9W6ds65KAhACtR93GToi"),
};

async function initializeAll() {
    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         Initializing All Tachyon L2 Contracts                        ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝\n');

    console.log(`Deployer: ${deployerKeypair.publicKey.toBase58()}`);
    console.log(`RPC: ${RPC_URL}\n`);

    // 1. Initialize TachyonStateCompression
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('1/6: Initializing TachyonStateCompression');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const { IDL } = require('../target/types/tachyon_state_compression');
        const program = new Program(IDL, provider) as any;
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("l2-state")], program.programId);
        
        try {
            const state = await program.account.l2State.fetch(statePda);
            console.log('⚠️  Already initialized\n');
        } catch {
            const tx = await program.methods
                .initialize(deployerKeypair.publicKey)
                .rpc();
            console.log(`✅ Initialized! TX: ${tx}\n`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}\n`);
    }

    // 2. Initialize TachyonL2Core
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('2/6: Initializing TachyonL2Core');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const { IDL } = require('../target/types/tachyon_l2_core');
        const program = new Program(IDL, provider) as any;
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("l2-core")], program.programId);
        
        try {
            const state = await program.account.l2CoreState.fetch(statePda);
            console.log('⚠️  Already initialized\n');
        } catch {
            const tx = await program.methods
                .initialize(
                    deployerKeypair.publicKey,
                    CONTRACTS.stateCompression,
                    CONTRACTS.verifier,
                    CONTRACTS.sequencer
                )
                .rpc();
            console.log(`✅ Initialized! TX: ${tx}\n`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}\n`);
    }

    // 3. TachyonVerifier (no initialization needed - stateless)
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('3/6: TachyonVerifier');
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('✅ No initialization needed (stateless)\n');

    // 4. Initialize TachyonBridge
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('4/6: Initializing TachyonBridge');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const { IDL } = require('../target/types/tachyon_bridge');
        const program = new Program(IDL, provider) as any;
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("bridge")], program.programId);
        
        try {
            const state = await program.account.bridgeState.fetch(statePda);
            console.log('⚠️  Already initialized\n');
        } catch {
            const tx = await program.methods
                .initialize(deployerKeypair.publicKey, [1, 56, 137]) // X1, BSC, Polygon
                .rpc();
            console.log(`✅ Initialized! TX: ${tx}\n`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}\n`);
    }

    // 5. Initialize TachyonSequencer
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('5/6: Initializing TachyonSequencer');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const { IDL } = require('../target/types/tachyon_sequencer');
        const program = new Program(IDL, provider) as any;
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("sequencer")], program.programId);
        
        try {
            const state = await program.account.sequencerState.fetch(statePda);
            console.log('⚠️  Already initialized\n');
        } catch {
            const minStake = new BN(100_000_000_000_000); // 100k TACH with 9 decimals
            const tx = await program.methods
                .initialize(deployerKeypair.publicKey, minStake)
                .rpc();
            console.log(`✅ Initialized! TX: ${tx}\n`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}\n`);
    }

    // 6. Initialize TachyonGovernance V2
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('6/6: Initializing TachyonGovernance V2');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const { IDL } = require('../target/types/tachyon_governance');
        const program = new Program(IDL, provider) as any;
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("governance")], program.programId);
        
        try {
            const state = await program.account.governanceState.fetch(statePda);
            console.log('⚠️  Already initialized\n');
        } catch {
            const minStake = new BN(100_000_000_000_000); // 100k TACH
            const minProposalStake = new BN(10_000_000_000_000); // 10k TACH
            const votingPeriod = new BN(604800); // 7 days
            
            const tx = await program.methods
                .initialize(minStake, minProposalStake, votingPeriod)
                .rpc();
            console.log(`✅ Initialized! TX: ${tx}\n`);
            
            // Get vault and rewards pool addresses
            const [vault] = PublicKey.findProgramAddressSync([Buffer.from("vault")], program.programId);
            const [rewardsPool] = PublicKey.findProgramAddressSync([Buffer.from("rewards-pool")], program.programId);
            
            console.log(`Vault: ${vault.toBase58()}`);
            console.log(`Rewards Pool: ${rewardsPool.toBase58()}\n`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}\n`);
    }

    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         ✅ Initialization Complete!                                  ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝\n');
    
    console.log('Next steps:');
    console.log('1. Fund rewards pool with 300M TACH');
    console.log('2. Transfer 100k TACH to node wallet');
    console.log('3. Stake and start node\n');
}

initializeAll().catch(console.error);

