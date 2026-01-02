const anchor = require('@coral-xyz/anchor');
const { PublicKey, Keypair, SystemProgram } = require('@solana/web3.js');
const fs = require('fs');

async function main() {
    console.log("\n╔══════════════════════════════════════════════════════════════════════╗");
    console.log("║         Initializing TachyonSequencer                                ║");
    console.log("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Load deployer keypair
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync('/root/.config/tachyon/deployer.json', 'utf-8')))
    );

    console.log("Deployer:", deployerKeypair.publicKey.toString());

    // Setup connection
    const connection = new anchor.web3.Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
    
    // Program ID
    const programId = new PublicKey('SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M');
    
    console.log("Program ID:", programId.toString());

    // Derive sequencer state PDA (must match contract seeds!)
    const [sequencerStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('sequencer')],  // Contract uses b"sequencer", not b"sequencer-state"
        programId
    );

    console.log("Sequencer State PDA:", sequencerStatePDA.toString());
    console.log("");

    // Check if already initialized
    try {
        const accountInfo = await connection.getAccountInfo(sequencerStatePDA);
        if (accountInfo && accountInfo.data.length > 0) {
            console.log("✅ Sequencer already initialized!");
            console.log("   Data length:", accountInfo.data.length);
            return;
        }
    } catch (e) {
        // Not initialized, continue
    }

    console.log("Initializing sequencer...");
    console.log("Parameters:");
    console.log("  Min Stake: 100,000 TACH");
    console.log("");

    // Build instruction manually
    // Discriminator for "initialize" - sha256("global:initialize")[0..8]
    const discriminator = Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]);
    
    // Parameters
    const authority = deployerKeypair.publicKey.toBuffer();
    const minStake = BigInt(100000 * 1e9); // 100,000 TACH (9 decimals)
    
    // Build data: discriminator + authority (32 bytes) + min_stake (u64)
    const data = Buffer.alloc(8 + 32 + 8);
    discriminator.copy(data, 0);
    authority.copy(data, 8);
    data.writeBigUInt64LE(minStake, 40);

    const instruction = new anchor.web3.TransactionInstruction({
        keys: [
            { pubkey: sequencerStatePDA, isSigner: false, isWritable: true },
            { pubkey: deployerKeypair.publicKey, isSigner: true, isWritable: true },
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: programId,
        data: data,
    });

    console.log("Sending transaction...");

    try {
        const tx = new anchor.web3.Transaction().add(instruction);
        const signature = await anchor.web3.sendAndConfirmTransaction(
            connection,
            tx,
            [deployerKeypair],
            { commitment: 'confirmed' }
        );

        console.log("\n✅ Sequencer initialized successfully!");
        console.log("   Transaction:", signature);
        console.log("   Sequencer State PDA:", sequencerStatePDA.toString());
        console.log("");
        console.log("Parameters set:");
        console.log("   Authority:", deployerKeypair.publicKey.toString());
        console.log("   Min Stake: 100,000 TACH");
        
        // Verify
        const accountInfo = await connection.getAccountInfo(sequencerStatePDA);
        if (accountInfo) {
            console.log("\n✅ Verification: Account exists!");
            console.log("   Data length:", accountInfo.data.length, "bytes");
        }
    } catch (error) {
        console.error("\n❌ Failed to initialize:", error.message);
        if (error.logs) {
            console.error("\nProgram logs:");
            error.logs.forEach(log => console.error("  ", log));
        }
        process.exit(1);
    }
}

main().catch(console.error);

