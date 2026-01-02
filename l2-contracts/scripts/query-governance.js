const anchor = require('@coral-xyz/anchor');
const { PublicKey } = require('@solana/web3.js');

async function queryGovernance() {
    try {
        const connection = new anchor.web3.Connection('https://rpc.mainnet.x1.xyz', 'confirmed');
        const governanceId = new PublicKey('TACHaFSmJJ1i6UXR1KkjxSP9W6ds65KAhACtR93GToi');
        
        // Derive governance state PDA
        const [governanceStatePDA] = PublicKey.findProgramAddressSync(
            [Buffer.from('governance')],
            governanceId
        );
        
        // Fetch account
        const accountInfo = await connection.getAccountInfo(governanceStatePDA);
        
        if (!accountInfo) {
            console.log("NOT_INITIALIZED");
            return;
        }
        
        // Parse data (simplified - just read key fields)
        const data = accountInfo.data;
        
        // Skip discriminator (8 bytes), read fields
        // authority: 32 bytes
        // tach_mint: 32 bytes
        // vault: 32 bytes
        // rewards_pool: 32 bytes
        // min_stake: 8 bytes (u64) at offset 136
        // total_staked: 8 bytes (u64) at offset ~160
        
        const totalStaked = data.readBigUInt64LE(160);
        const minStake = data.readBigUInt64LE(136);
        
        // Convert from lamports (base units with 9 decimals) to TACH tokens
        const totalStakedTACH = Number(totalStaked) / 1e9;
        const minStakeTACH = Number(minStake) / 1e9;
        
        console.log(`TOTAL_STAKED:${totalStakedTACH}`);
        console.log(`MIN_STAKE:${minStakeTACH}`);
        console.log(`ACTIVE_NODES:1`); // For now, count active nodes as 1
        
    } catch (error) {
        console.log("ERROR:" + error.message);
    }
}

queryGovernance();

