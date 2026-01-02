import * as anchor from '@coral-xyz/anchor';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';

const RPC_URL = 'https://rpc.mainnet.x1.xyz';

// Contract addresses
const CONTRACTS = [
    {
        name: 'TachyonStateCompression',
        programId: 'L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx',
        idlPath: '../target/idl/tachyon_state_compression.json'
    },
    {
        name: 'TachyonL2Core',
        programId: 'CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3',
        idlPath: '../target/idl/tachyon_l2_core.json'
    },
    {
        name: 'TachyonVerifier',
        programId: 'VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR',
        idlPath: '../target/idl/tachyon_verifier.json'
    },
    {
        name: 'TachyonBridge',
        programId: 'BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW',
        idlPath: '../target/idl/tachyon_bridge.json'
    },
    {
        name: 'TachyonSequencer',
        programId: 'SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M',
        idlPath: '../target/idl/tachyon_sequencer.json'
    },
    {
        name: 'TachyonGovernance',
        programId: 'TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9',
        idlPath: '../target/idl/tachyon_governance.json'
    }
];

async function initializeContract(
    name: string,
    programIdStr: string,
    idlPath: string,
    provider: AnchorProvider
) {
    console.log('═══════════════════════════════════════════════════════════════════════');
    console.log(`Initializing ${name} (${programIdStr})...`);
    console.log('═══════════════════════════════════════════════════════════════════════');
    
    try {
        const programId = new PublicKey(programIdStr);
        
        // Load IDL
        const idlFullPath = path.join(__dirname, idlPath);
        if (!fs.existsSync(idlFullPath)) {
            console.log(`⚠️  IDL file not found: ${idlFullPath}`);
            return;
        }
        
        const idl = JSON.parse(fs.readFileSync(idlFullPath, 'utf-8'));
        const program = new Program(idl, provider);
        
        // Check if contract has initialize instruction
        const initInstruction = idl.instructions.find((ix: any) => 
            ix.name === 'initialize' || ix.name === 'init'
        );
        
        if (!initInstruction) {
            console.log('⚠️  No initialize instruction found');
            console.log('✅ This contract may not need initialization');
            console.log('');
            return;
        }
        
        // Derive state PDA
        const [statePda] = PublicKey.findProgramAddressSync(
            [Buffer.from('l2_state')],
            program.programId
        );
        
        console.log('Authority:', provider.wallet.publicKey.toBase58());
        console.log('State PDA:', statePda.toBase58());
        
        // Build accounts object with correct names (camelCase for Anchor)
        const accountNames: any = {
            payer: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        };
        
        // Add the state account with its specific name (convert snake_case to camelCase)
        const stateAccountName = initInstruction.accounts.find((acc: any) => 
            acc.name.includes('state') || acc.name.includes('State')
        )?.name;
        
        if (stateAccountName) {
            // Convert snake_case to camelCase
            const camelCaseName = stateAccountName.replace(/_([a-z])/g, (g: string) => g[1].toUpperCase());
            accountNames[camelCaseName] = statePda;
        }
        
        console.log('Accounts:', JSON.stringify(accountNames, null, 2));
        
        // Try to initialize
        try {
            const tx = await program.methods
                .initialize()
                .accountsPartial(accountNames)
                .signers([])
                .rpc();
            
            console.log('✅ Initialized successfully!');
            console.log('Transaction:', tx);
        } catch (error: any) {
            if (error.message?.includes('already in use') || 
                error.message?.includes('already initialized') ||
                error.logs?.some((log: string) => log.includes('already in use'))) {
                console.log('✅ Already initialized');
            } else {
                console.log('❌ Error:', error.message);
                if (error.logs) {
                    console.log('Logs:', error.logs.join('\n'));
                }
            }
        }
        
    } catch (error: any) {
        console.log('❌ Failed to initialize:', error.message);
    }
    
    console.log('');
}

async function main() {
    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         L2 Contract Initialization                                   ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝');
    console.log('');
    
    // Load deployer keypair
    const deployerPath = process.env.DEPLOYER_KEYPAIR || '/root/deployer.json';
    
    if (!fs.existsSync(deployerPath)) {
        console.error('❌ Deployer keypair not found:', deployerPath);
        console.error('Set DEPLOYER_KEYPAIR environment variable or place deployer.json in /root/');
        process.exit(1);
    }
    
    const deployerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(deployerPath, 'utf-8')))
    );
    
    console.log('Authority:', deployerKeypair.publicKey.toBase58());
    console.log('RPC:', RPC_URL);
    console.log('');
    
    // Setup connection and provider
    const connection = new Connection(RPC_URL, 'confirmed');
    const wallet = new Wallet(deployerKeypair);
    const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
    
    // Initialize each contract
    for (const contract of CONTRACTS) {
        await initializeContract(
            contract.name,
            contract.programId,
            contract.idlPath,
            provider
        );
    }
    
    console.log('╔══════════════════════════════════════════════════════════════════════╗');
    console.log('║         ✅ Initialization complete!                                  ║');
    console.log('╚══════════════════════════════════════════════════════════════════════╝');
}

main().catch((error) => {
    console.error('Fatal error:', error);
    process.exit(1);
});

