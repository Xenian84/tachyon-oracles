const anchor = require('@coral-xyz/anchor');
const { PublicKey, Keypair, SystemProgram } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } = require('@solana/spl-token');
const fs = require('fs');

async function main() {
    console.log("\n╔══════════════════════════════════════════════════════════════════════╗");
    console.log("║         Initializing TachyonGovernance                               ║");
    console.log("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Load deployer keypair
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync('/root/.config/tachyon/deployer.json', 'utf-8')))
    );

    console.log("Deployer:", deployerKeypair.publicKey.toString());

    // Setup connection
    const connection = new anchor.web3.Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
    
    // Program ID
    const programId = new PublicKey('TACHaFSmJJ1i6UXR1KkjxSP9W6ds65KAhACtR93GToi');
    const tachMint = new PublicKey('TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu');
    
    console.log("Program ID:", programId.toString());
    console.log("TACH Mint:", tachMint.toString());

    // Derive PDAs
    const [governanceStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('governance')],
        programId
    );
    
    const [vaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('vault')],
        programId
    );
    
    const [rewardsPoolPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('rewards-pool')],
        programId
    );

    console.log("Governance State PDA:", governanceStatePDA.toString());
    console.log("Vault PDA:", vaultPDA.toString());
    console.log("Rewards Pool PDA:", rewardsPoolPDA.toString());
    console.log("");

    // Check if already initialized
    try {
        const accountInfo = await connection.getAccountInfo(governanceStatePDA);
        if (accountInfo && accountInfo.data.length > 0) {
            console.log("✅ Governance already initialized!");
            console.log("   Data length:", accountInfo.data.length);
            return;
        }
    } catch (e) {
        // Not initialized, continue
    }

    console.log("Initializing governance...");
    console.log("Parameters:");
    console.log("  Min Stake: 100,000 TACH");
    console.log("  Min Proposal Stake: 10,000 TACH");
    console.log("  Voting Period: 259,200 seconds (3 days)");
    console.log("");

    // Build instruction manually
    // Discriminator for "initialize" - sha256("global:initialize")[0..8]
    const discriminator = Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]);
    
    // Parameters
    const minStake = BigInt(100000 * 1e9); // 100,000 TACH (9 decimals)
    const minProposalStake = BigInt(10000 * 1e9); // 10,000 TACH
    const votingPeriod = BigInt(259200); // 3 days in seconds
    
    // Build data: discriminator + min_stake (u64) + min_proposal_stake (u64) + voting_period (i64)
    const data = Buffer.alloc(8 + 8 + 8 + 8);
    discriminator.copy(data, 0);
    data.writeBigUInt64LE(minStake, 8);
    data.writeBigUInt64LE(minProposalStake, 16);
    data.writeBigInt64LE(votingPeriod, 24);

    // Account order must EXACTLY match the Initialize struct in the contract:
    // governance_state, vault, rewards_pool, tach_mint, authority, payer,
    // token_program, associated_token_program, system_program, rent
    const instruction = new anchor.web3.TransactionInstruction({
        keys: [
            { pubkey: governanceStatePDA, isSigner: false, isWritable: true },  // 0: governance_state
            { pubkey: vaultPDA, isSigner: false, isWritable: true },            // 1: vault
            { pubkey: rewardsPoolPDA, isSigner: false, isWritable: true },      // 2: rewards_pool
            { pubkey: tachMint, isSigner: false, isWritable: false },           // 3: tach_mint
            { pubkey: deployerKeypair.publicKey, isSigner: true, isWritable: false }, // 4: authority
            { pubkey: deployerKeypair.publicKey, isSigner: true, isWritable: true },  // 5: payer
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },   // 6: token_program
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // 7: system_program
            { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }, // 8: rent
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

        console.log("\n✅ Governance initialized successfully!");
        console.log("   Transaction:", signature);
        console.log("   Governance State PDA:", governanceStatePDA.toString());
        console.log("   Vault PDA:", vaultPDA.toString());
        console.log("   Rewards Pool PDA:", rewardsPoolPDA.toString());
        console.log("");
        console.log("Parameters set:");
        console.log("   Min Stake: 100,000 TACH");
        console.log("   Min Proposal Stake: 10,000 TACH");
        console.log("   Voting Period: 3 days");
        
        // Verify
        const accountInfo = await connection.getAccountInfo(governanceStatePDA);
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

