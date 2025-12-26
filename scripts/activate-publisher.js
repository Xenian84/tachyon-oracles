const { Connection, Keypair, PublicKey, Transaction, TransactionInstruction } = require('@solana/web3.js');
const fs = require('fs');
const crypto = require('crypto');

const PROGRAM_ID = new PublicKey('TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1');
const RPC_URL = 'https://rpc.mainnet.x1.xyz';

async function activatePublisher(publisherKeyPath) {
    console.log('\nActivating publisher...\n');
    
    // Load authority (deployer)
    const authorityData = JSON.parse(fs.readFileSync('/root/.config/solana/deployer.json'));
    const authority = Keypair.fromSecretKey(new Uint8Array(authorityData));
    
    console.log('Authority:', authority.publicKey.toString());
    
    // Load publisher keypair to get public key
    const publisherData = JSON.parse(fs.readFileSync(publisherKeyPath));
    const publisherKeypair = Keypair.fromSecretKey(new Uint8Array(publisherData));
    const publisherKey = publisherKeypair.publicKey;
    
    console.log('Publisher:', publisherKey.toString());
    
    // Derive PDAs
    const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('config')],
        PROGRAM_ID
    );
    
    const [publisherPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('publisher'), publisherKey.toBuffer()],
        PROGRAM_ID
    );
    
    console.log('Config PDA:', configPda.toString());
    console.log('Publisher PDA:', publisherPda.toString());
    console.log('');
    
    // Create connection
    const connection = new Connection(RPC_URL, 'confirmed');
    
    // Check if publisher exists
    const publisherInfo = await connection.getAccountInfo(publisherPda);
    if (!publisherInfo) {
        console.log('❌ Publisher not registered. Register first with:');
        console.log(`   node scripts/register-publisher-simple.js ${publisherKeyPath}`);
        process.exit(1);
    }
    
    // Check current status
    const currentStatus = publisherInfo.data[40];
    if (currentStatus === 1) {
        console.log('✅ Publisher is already ACTIVE');
        process.exit(0);
    }
    
    console.log('Current status: INACTIVE');
    console.log('Activating...\n');
    
    // Build set_publisher_status instruction
    // Instruction discriminator for "set_publisher_status"
    const discriminator = Buffer.from(
        crypto.createHash('sha256')
            .update('global:set_publisher_status')
            .digest()
    ).slice(0, 8);
    
    // Instruction data: discriminator + is_active (1 byte, true = 1)
    const instructionData = Buffer.concat([
        discriminator,
        Buffer.from([1]) // is_active = true
    ]);
    
    const instruction = new TransactionInstruction({
        keys: [
            { pubkey: configPda, isSigner: false, isWritable: false }, // config (read-only)
            { pubkey: publisherPda, isSigner: false, isWritable: true }, // publisher_account (mut)
            { pubkey: authority.publicKey, isSigner: true, isWritable: false }, // admin (signer)
        ],
        programId: PROGRAM_ID,
        data: instructionData,
    });
    
    // Create and send transaction
    const transaction = new Transaction().add(instruction);
    transaction.feePayer = authority.publicKey;
    
    console.log('Sending transaction...');
    const signature = await connection.sendTransaction(transaction, [authority]);
    
    console.log('Transaction sent:', signature);
    console.log('Waiting for confirmation...\n');
    
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log('✅ Publisher activated successfully!');
    console.log('Transaction signature:', signature);
    console.log('Publisher PDA:', publisherPda.toString());
    console.log('');
    console.log('The publisher can now submit price updates!');
    console.log('');
}

// Main
const publisherKeyPath = process.argv[2] || '/root/tachyon-oracles/keys/validator1-signer.json';

activatePublisher(publisherKeyPath).catch(err => {
    console.error('Error:', err.message);
    process.exit(1);
});

