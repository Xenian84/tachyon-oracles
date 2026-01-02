const anchor = require('@coral-xyz/anchor');
const { PublicKey, Keypair } = require('@solana/web3.js');
const fs = require('fs');

async function main() {
    console.log("\n╔══════════════════════════════════════════════════════════════════════╗");
    console.log("║         Updating L2 State Authority                                 ║");
    console.log("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Load deployer keypair (current authority)
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync('/root/.config/tachyon/deployer.json', 'utf-8')))
    );

    // New authority (node wallet)
    const newAuthority = new PublicKey('3zW9uNHEqhBUYAxGbR5MnAhd6Q38husgymAheF2ZV5CN');

    console.log("Current Authority (deployer):", deployerKeypair.publicKey.toString());
    console.log("New Authority (node wallet):", newAuthority.toString());
    console.log("");

    // Setup connection
    const connection = new anchor.web3.Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
    
    // Program ID
    const programId = new PublicKey('L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx');

    // Derive L2 state PDA
    const [l2StatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('l2-state')],
        programId
    );

    console.log("L2 State PDA:", l2StatePDA.toString());
    console.log("");

    // Read current state
    const accountInfo = await connection.getAccountInfo(l2StatePDA);
    if (!accountInfo) {
        console.error("❌ L2 State account not found!");
        process.exit(1);
    }

    // Parse the account data to show current authority
    const currentAuthority = new PublicKey(accountInfo.data.slice(8, 40)); // Skip discriminator, read pubkey
    console.log("Current authority in contract:", currentAuthority.toString());
    console.log("");

    // Build update_authority instruction
    // Discriminator for update_authority - sha256("global:update_authority")[0..8]
    const discriminator = Buffer.from([0x32, 0xc3, 0x5c, 0x6c, 0x93, 0x54, 0x7c, 0x6e]); // You'll need to calculate this
    const newAuthorityBytes = newAuthority.toBuffer();
    const data = Buffer.concat([discriminator, newAuthorityBytes]);

    console.log("⚠️  NOTE: The contract needs an update_authority function!");
    console.log("For now, let's just verify the current state and document the issue.");
    console.log("");
    console.log("WORKAROUND: Use the deployer wallet for the node instead!");
}

main().catch(console.error);

