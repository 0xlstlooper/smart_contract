import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SmartContract } from "../target/types/smart_contract";

describe("smart_contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SmartContract as Program<SmartContract>;
  const payer = provider.wallet.publicKey;

  it("Is initialized!", async () => {
    const startTick = new anchor.BN(100);
    const tickSize = new anchor.BN(10);

    const [configPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("config")],
      program.programId
    );

    const [allAssetsPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("allassets")],
      program.programId
    );

    const tx = await program.methods
      .initialize(startTick, tickSize)
      .accounts({
        payer: payer,
        config: configPda,
        allassets: allAssetsPda,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Your transaction signature", tx);
  });
});