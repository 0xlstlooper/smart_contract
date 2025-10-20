// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { SmartContract } from "../target/types/smart_contract";

// describe("smart_contract", () => {
//   // Configure the client to use the local cluster.
//   anchor.setProvider(anchor.AnchorProvider.env());

//   const program = anchor.workspace.smartContract as Program<SmartContract>;

//   it("Is initialized!", async () => {
//     const startTick = new anchor.BN(0);
//     const tickSize = new anchor.BN(1);
//     const tx = await program.methods.initialize(startTick, tickSize).rpc();
//     console.log("Your transaction signature", tx);
//   });
// });
