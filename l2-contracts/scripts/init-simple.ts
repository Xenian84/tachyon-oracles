import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import * as fs from 'fs';

const RPC_URL = 'https://rpc.mainnet.x1.xyz';
const connection = new Connection(RPC_URL, 'confirmed');

// Load deployer keypair
const deployerKeypairPath = process.env.DEPLOYER_KEYPAIR || '/root/.config/tachyon/deployer.json';
const deployerKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(deployerKeypairPath, 'utf-8')))
);

// Contract addresses
const CONTRACTS = {
    stateCompression: new PublicKey("L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx"),
    l2Core: new PublicKey("CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3"),
    verifier: new PublicKey("VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR"),
    bridge: new PublicKey("BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW"),
    sequencer: new PublicKey("SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M"),
    governance: new PublicKey("TACHaFSmJJ1i6UXR1KkjxSP9W6ds65KAhACtR93GToi"),
};

async function checkInitialization() {
    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         Checking L2 Contract Initialization Status                   ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝\n');

    console.log(`Deployer: ${deployerKeypair.publicKey.toBase58()}`);
    console.log(`RPC: ${RPC_URL}\n`);

    // 1. Check TachyonStateCompression
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('1/6: TachyonStateCompression');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("l2-state")], CONTRACTS.stateCompression);
        const accountInfo = await connection.getAccountInfo(statePda);
        
        if (accountInfo) {
            console.log(`✅ Initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
            console.log(`   Data size: ${accountInfo.data.length} bytes`);
        } else {
            console.log(`⚠️  NOT initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}`);
    }
    console.log();

    // 2. Check TachyonL2Core
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('2/6: TachyonL2Core');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("l2-core")], CONTRACTS.l2Core);
        const accountInfo = await connection.getAccountInfo(statePda);
        
        if (accountInfo) {
            console.log(`✅ Initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
            console.log(`   Data size: ${accountInfo.data.length} bytes`);
        } else {
            console.log(`⚠️  NOT initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}`);
    }
    console.log();

    // 3. TachyonVerifier (stateless)
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('3/6: TachyonVerifier');
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('✅ No initialization needed (stateless verifier)\n');

    // 4. Check TachyonBridge
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('4/6: TachyonBridge');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("bridge")], CONTRACTS.bridge);
        const accountInfo = await connection.getAccountInfo(statePda);
        
        if (accountInfo) {
            console.log(`✅ Initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
            console.log(`   Data size: ${accountInfo.data.length} bytes`);
        } else {
            console.log(`⚠️  NOT initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}`);
    }
    console.log();

    // 5. Check TachyonSequencer
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('5/6: TachyonSequencer');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("sequencer")], CONTRACTS.sequencer);
        const accountInfo = await connection.getAccountInfo(statePda);
        
        if (accountInfo) {
            console.log(`✅ Initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
            console.log(`   Data size: ${accountInfo.data.length} bytes`);
        } else {
            console.log(`⚠️  NOT initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}`);
    }
    console.log();

    // 6. Check TachyonGovernance V2
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log('6/6: TachyonGovernance V2');
    console.log('═══════════════════════════════════════════════════════════════════════');
    try {
        const [statePda] = PublicKey.findProgramAddressSync([Buffer.from("governance")], CONTRACTS.governance);
        const [vault] = PublicKey.findProgramAddressSync([Buffer.from("vault")], CONTRACTS.governance);
        const [rewardsPool] = PublicKey.findProgramAddressSync([Buffer.from("rewards-pool")], CONTRACTS.governance);
        
        const accountInfo = await connection.getAccountInfo(statePda);
        
        if (accountInfo) {
            console.log(`✅ Initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
            console.log(`   Data size: ${accountInfo.data.length} bytes`);
            console.log(`   Token Vault: ${vault.toBase58()}`);
            console.log(`   Rewards Pool: ${rewardsPool.toBase58()}`);
        } else {
            console.log(`⚠️  NOT initialized`);
            console.log(`   State PDA: ${statePda.toBase58()}`);
            console.log(`   Token Vault: ${vault.toBase58()}`);
            console.log(`   Rewards Pool: ${rewardsPool.toBase58()}`);
        }
    } catch (e: any) {
        console.log(`❌ Error: ${e.message}`);
    }
    console.log();

    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         Status Check Complete!                                       ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝\n');
    
    console.log('📝 Note: Most contracts use lazy initialization.');
    console.log('   They will be initialized automatically on first use.\n');
    
    console.log('Next steps:');
    console.log('1. Fund rewards pool with 300M TACH');
    console.log('2. Transfer 100k TACH to node wallet');
    console.log('3. Stake and start node\n');
}

checkInitialization().catch(console.error);

