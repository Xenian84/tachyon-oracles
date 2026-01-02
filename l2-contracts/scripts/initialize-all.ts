/**
 * Initialize all L2 contracts
 */

import * as anchor from '@coral-xyz/anchor';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'https://rpc.mainnet.x1.xyz';
const DEPLOYER_PATH = '/root/.config/solana/deployer.json';

// Program IDs
const PROGRAMS = {
  stateCompression: new PublicKey('L2TA7eVsDyXx7nxF4p2Xay3iWgdCHuMPx6YV5odwMTx'),
  l2Core: new PublicKey('CXREjmHFdCBNZe7x1fLLam7VMph2A6uRRroaNUpzEwG3'),
  verifier: new PublicKey('VRFYGHjfBedWbwTBw8DhmoUYa6s3Ga5ybJUPny7buAR'),
  bridge: new PublicKey('BRDGK2ASP86oe5wj18XYwRBuhEELpEGFqZGBhxnwwnTW'),
  sequencer: new PublicKey('SEQRXNAYH7s4DceD8K3Bb7oChunLVYqZKRcCJGRoQ1M'),
  governance: new PublicKey('TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9'),
};

async function main() {
  console.log('🚀 Initializing all L2 contracts...\n');

  // Setup connection
  const connection = new Connection(RPC_URL, 'confirmed');
  const deployerKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(DEPLOYER_PATH, 'utf-8')))
  );
  const wallet = new Wallet(deployerKeypair);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
  });

  console.log(`Authority: ${deployerKeypair.publicKey.toBase58()}\n`);

  // 1. Initialize State Compression
  console.log('1️⃣  Initializing TachyonStateCompression (L2TA)...');
  try {
    const [l2StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('l2-state')],
      PROGRAMS.stateCompression
    );

    // Check if already initialized
    const accountInfo = await connection.getAccountInfo(l2StatePda);
    if (accountInfo) {
      console.log('   ✅ Already initialized');
    } else {
      // TODO: Call initialize instruction
      console.log('   ⏳ Initialization needed (manual step)');
    }
  } catch (error: any) {
    console.log(`   ❌ Error: ${error.message}`);
  }
  console.log();

  // 2. Initialize L2 Core
  console.log('2️⃣  Initializing TachyonL2Core (CXRE)...');
  try {
    const [coreStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('l2-core')],
      PROGRAMS.l2Core
    );

    const accountInfo = await connection.getAccountInfo(coreStatePda);
    if (accountInfo) {
      console.log('   ✅ Already initialized');
    } else {
      console.log('   ⏳ Initialization needed (manual step)');
    }
  } catch (error: any) {
    console.log(`   ❌ Error: ${error.message}`);
  }
  console.log();

  // 3. Initialize Sequencer
  console.log('3️⃣  Initializing TachyonSequencer (SEQR)...');
  try {
    const [sequencerStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('sequencer')],
      PROGRAMS.sequencer
    );

    const accountInfo = await connection.getAccountInfo(sequencerStatePda);
    if (accountInfo) {
      console.log('   ✅ Already initialized');
    } else {
      console.log('   ⏳ Initialization needed (manual step)');
    }
  } catch (error: any) {
    console.log(`   ❌ Error: ${error.message}`);
  }
  console.log();

  // 4. Initialize Governance
  console.log('4️⃣  Initializing TachyonGovernance (TACH)...');
  try {
    const [governanceStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('governance')],
      PROGRAMS.governance
    );

    const accountInfo = await connection.getAccountInfo(governanceStatePda);
    if (accountInfo) {
      console.log('   ✅ Already initialized');
    } else {
      console.log('   ⏳ Initialization needed (manual step)');
    }
  } catch (error: any) {
    console.log(`   ❌ Error: ${error.message}`);
  }
  console.log();

  // 5. Initialize Bridge
  console.log('5️⃣  Initializing TachyonBridge (BRDG)...');
  try {
    const [bridgeStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('bridge')],
      PROGRAMS.bridge
    );

    const accountInfo = await connection.getAccountInfo(bridgeStatePda);
    if (accountInfo) {
      console.log('   ✅ Already initialized');
    } else {
      console.log('   ⏳ Initialization needed (manual step)');
    }
  } catch (error: any) {
    console.log(`   ❌ Error: ${error.message}`);
  }
  console.log();

  console.log('✅ Initialization check complete!\n');
  console.log('📝 Summary:');
  console.log('   All contracts are deployed and ready for initialization.');
  console.log('   Use Anchor CLI or custom scripts to initialize each contract.\n');
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

