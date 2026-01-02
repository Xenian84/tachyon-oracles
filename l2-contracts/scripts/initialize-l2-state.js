const anchor = require('@coral-xyz/anchor');
const { PublicKey, Keypair, SystemProgram } = require('@solana/web3.js');
const fs = require('fs');

async function main() {
    console.log("\n╔══════════════════════════════════════════════════════════════════════╗");
    console.log("║         Initializing L2 State Account                               ║");
    console.log("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Load deployer keypair
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync('/root/.config/tachyon/deployer.json', 'utf-8')))
    );

    console.log("Deployer:", deployerKeypair.publicKey.toString());

    // Setup connection
    const connection = new anchor.web3.Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
    const wallet = new anchor.Wallet(deployerKeypair);
    const provider = new anchor.AnchorProvider(connection, wallet, { commitment: 'confirmed' });
    anchor.setProvider(provider);

    // Load program
    const programId = new PublicKey('L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx');
    const idl = JSON.parse(fs.readFileSync('/root/tachyon-oracles/l2-contracts/target/idl/tachyon_state_compression.json', 'utf-8'));
    const program = new anchor.Program(idl, programId, provider);

    console.log("Program ID:", programId.toString());

    // Derive L2 state PDA
    const [l2StatePDA, bump] = await PublicKey.findProgramAddress(
        [Buffer.from('l2-state')],
        programId
    );

    console.log("L2 State PDA:", l2StatePDA.toString());
    console.log("Bump:", bump);
    console.log("");

    // Check if already initialized
    try {
        const accountInfo = await connection.getAccountInfo(l2StatePDA);
        if (accountInfo) {
            console.log("✅ L2 State account already exists!");
            console.log("   Data length:", accountInfo.data.length);
            console.log("   Owner:", accountInfo.owner.toString());
            return;
        }
    } catch (e) {
        // Account doesn't exist, continue to initialize
    }

    console.log("Initializing L2 State account...");

    try {
        const tx = await program.methods
            .initialize(deployerKeypair.publicKey)
            .accounts({
                l2State: l2StatePDA,
                payer: deployerKeypair.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        console.log("\n✅ L2 State initialized successfully!");
        console.log("   Transaction:", tx);
        console.log("   L2 State PDA:", l2StatePDA.toString());
        console.log("   Authority:", deployerKeypair.publicKey.toString());
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

