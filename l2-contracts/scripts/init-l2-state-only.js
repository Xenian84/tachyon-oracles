const anchor = require('@coral-xyz/anchor');
const { PublicKey, Keypair, SystemProgram } = require('@solana/web3.js');
const fs = require('fs');

async function main() {
    console.log("\n╔══════════════════════════════════════════════════════════════════════╗");
    console.log("║         Initializing ONLY TachyonStateCompression                   ║");
    console.log("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Load deployer keypair
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync('/root/.config/tachyon/deployer.json', 'utf-8')))
    );

    console.log("Deployer:", deployerKeypair.publicKey.toString());

    // Setup connection
    const connection = new anchor.web3.Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
    
    // Program ID
    const programId = new PublicKey('L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx');
    console.log("Program ID:", programId.toString());

    // Derive L2 state PDA
    const [l2StatePDA, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from('l2-state')],
        programId
    );

    console.log("L2 State PDA:", l2StatePDA.toString());
    console.log("Bump:", bump);
    console.log("");

    // Check if already initialized
    try {
        const accountInfo = await connection.getAccountInfo(l2StatePDA);
        if (accountInfo && accountInfo.data.length > 0) {
            console.log("✅ L2 State account already exists!");
            console.log("   Data length:", accountInfo.data.length);
            console.log("   Owner:", accountInfo.owner.toString());
            return;
        }
    } catch (e) {
        // Account doesn't exist, continue
    }

    console.log("Creating initialize instruction...");

    // Build instruction manually
    // Instruction discriminator for "initialize" - first 8 bytes of sha256("global:initialize")
    const discriminator = Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]); // This is the Anchor discriminator
    
    // Authority pubkey (32 bytes)
    const authorityBytes = deployerKeypair.publicKey.toBuffer();
    
    // Combine discriminator + authority
    const data = Buffer.concat([discriminator, authorityBytes]);

    const instruction = new anchor.web3.TransactionInstruction({
        keys: [
            { pubkey: l2StatePDA, isSigner: false, isWritable: true },
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

        console.log("\n✅ L2 State initialized successfully!");
        console.log("   Transaction:", signature);
        console.log("   L2 State PDA:", l2StatePDA.toString());
        console.log("   Authority:", deployerKeypair.publicKey.toString());
        
        // Verify
        const accountInfo = await connection.getAccountInfo(l2StatePDA);
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

