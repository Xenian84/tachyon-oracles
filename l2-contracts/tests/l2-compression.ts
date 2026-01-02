import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { L2Compression } from "../target/types/l2_compression";

describe("l2-compression", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.l2Compression as Program<L2Compression>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
