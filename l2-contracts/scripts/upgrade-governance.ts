import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import * as fs from 'fs';
import { IDL as TachyonGovernanceIDL } from '../target/types/tachyon_governance';

const RPC_URL = 'https://rpc.mainnet.x1.xyz';
const connection = new Connection(RPC_URL, 'confirmed');

// Load deployer keypair
const deployerKeypairPath = process.env.DEPLOYER_KEYPAIR || '/root/.config/tachyon/deployer.json';
const deployerKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(deployerKeypairPath, 'utf-8')))
);

const provider = new AnchorProvider(connection, new Wallet(deployerKeypair), { commitment: 'confirmed' });

const GOVERNANCE_PROGRAM_ID = new PublicKey("TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9");
const TACH_MINT = new PublicKey("TACHrJvY9k4xn147mewGUiA2C6f19Wjtf91V5S6F5nu");

async function upgradeGovernance() {
    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         Upgrading TachyonGovernance Contract                         ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝\n');

    console.log(`Deployer: ${deployerKeypair.publicKey.toBase58()}`);
    console.log(`Program ID: ${GOVERNANCE_PROGRAM_ID.toBase58()}`);
    console.log(`TACH Mint: ${TACH_MINT.toBase58()}\n`);

    // Step 1: Upgrade the program
    console.log('Step 1: Upgrading program binary...');
    const { execSync } = require('child_process');
    
    try {
        const result = execSync(
            `solana program deploy /root/tachyon-oracles/l2-contracts/target/sbpf-solana-solana/release/tachyon_governance.so ` +
            `--program-id ${GOVERNANCE_PROGRAM_ID.toBase58()} ` +
            `--keypair ${deployerKeypairPath} ` +
            `--url ${RPC_URL}`,
            { encoding: 'utf-8' }
        );
        console.log(result);
        console.log('✅ Program upgraded successfully!\n');
    } catch (error) {
        console.error('❌ Program upgrade failed:', error);
        process.exit(1);
    }

    // Step 2: Check if governance state needs migration
    console.log('Step 2: Checking governance state...');
    const program = new Program(TachyonGovernanceIDL, GOVERNANCE_PROGRAM_ID, provider);
    
    const [governanceStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("governance")],
        program.programId
    );

    try {
        const governanceState = await program.account.governanceState.fetch(governanceStatePda);
        console.log('✅ Governance state exists');
        console.log(`   Authority: ${governanceState.authority.toBase58()}`);
        console.log(`   Total Staked: ${governanceState.totalStaked.toString()} TACH`);
        console.log(`   Total Proposals: ${governanceState.totalProposals.toString()}\n`);
        
        // Check if vault and rewards pool exist
        const [vaultPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault")],
            program.programId
        );
        
        const [rewardsPoolPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("rewards-pool")],
            program.programId
        );
        
        console.log('Checking vault and rewards pool...');
        const vaultInfo = await connection.getAccountInfo(vaultPda);
        const rewardsPoolInfo = await connection.getAccountInfo(rewardsPoolPda);
        
        if (!vaultInfo) {
            console.log('⚠️  Vault does not exist - needs initialization');
        } else {
            console.log('✅ Vault exists');
        }
        
        if (!rewardsPoolInfo) {
            console.log('⚠️  Rewards pool does not exist - needs initialization');
        } else {
            console.log('✅ Rewards pool exists');
        }
        
    } catch (error) {
        console.log('⚠️  Governance state not found - needs initialization\n');
    }

    console.log('\n╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         Upgrade Complete!                                            ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝');
    console.log('\nNext steps:');
    console.log('1. If vault/rewards pool missing, run: npx ts-node scripts/reinit-governance.ts');
    console.log('2. Fund rewards pool with 300M TACH');
    console.log('3. Test staking flow');
}

upgradeGovernance().catch(console.error);

