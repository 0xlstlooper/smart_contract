// Outdate: moved to total_flow.ts

// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { SmartContract } from "../target/types/smart_contract";
// import { BN } from "bn.js";
// import {
//   Keypair,
//   PublicKey,
//   SystemProgram,
// } from "@solana/web3.js";
// import {
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   TOKEN_2022_PROGRAM_ID,
//   TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccount,
//   createMint,
//   getAccount,
//   getAssociatedTokenAddressSync,
//   mintTo,
// } from "@solana/spl-token";
// import { assert } from "chai";

// describe("test deposit withdraw", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const payer = provider.wallet as anchor.Wallet;
//   const program = anchor.workspace.SmartContract as Program<SmartContract>;

//   // Constants from the program state
//   const MAX_ASSETS = 2;
//   const ORDERBOOK_SIZE = 10;
//   const START_TICK = new BN(100); // e.g., 1%
//   const TICK_SIZE = new BN(10); // e.g., 0.1%

//   // PDAs
//   let config: PublicKey;
//   let allAssets: PublicKey;
//   let vaultAuthority: PublicKey;

//   // Mints
//   let mintAsset1: Keypair;
//   let mintAsset2: Keypair;

//   // ATAs
//   let vaultAsset1: PublicKey;
//   let vaultAsset2: PublicKey;
//   let userAsset1TokenAccount: PublicKey;


//   before(async () => {
//     // Find PDAs
//     [config] = PublicKey.findProgramAddressSync(
//       [Buffer.from("config")],
//       program.programId
//     );
//     [allAssets] = PublicKey.findProgramAddressSync(
//       [Buffer.from("allassets")],
//       program.programId
//     );
//     [vaultAuthority] = PublicKey.findProgramAddressSync(
//       [Buffer.from("vault_authority")],
//       program.programId
//     );

//     // Create Mints for our assets
//     mintAsset1 = Keypair.generate();
//     mintAsset2 = Keypair.generate();

//     await createMint(
//       provider.connection,
//       payer.payer,
//       payer.publicKey,
//       null,
//       6,
//       mintAsset1,
//       undefined,
//       TOKEN_PROGRAM_ID
//     );

//     await createMint(
//       provider.connection,
//       payer.payer,
//       payer.publicKey,
//       null,
//       6,
//       mintAsset2,
//       undefined,
//       TOKEN_PROGRAM_ID
//     );

//     // Find ATA addresses for vaults
//     vaultAsset1 = getAssociatedTokenAddressSync(
//       mintAsset1.publicKey,
//       vaultAuthority,
//       true,
//       TOKEN_PROGRAM_ID
//     );
//     vaultAsset2 = getAssociatedTokenAddressSync(
//       mintAsset2.publicKey,
//       vaultAuthority,
//       true,
//       TOKEN_PROGRAM_ID
//     );
    
//     // Create user's ATA for Asset 1
//     userAsset1TokenAccount = await createAssociatedTokenAccount(
//         provider.connection,
//         payer.payer,
//         mintAsset1.publicKey,
//         payer.publicKey,
//         undefined,
//         TOKEN_PROGRAM_ID
//     );


//     // --- Initialize the Market ---
//     await program.methods
//       .initialize(START_TICK, TICK_SIZE)
//       .accounts({
//         payer: payer.publicKey,
//         config: config,
//         allassets: allAssets,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     // --- Add the First Asset ---
//     const multiplier1 = new BN(15);
//     await program.methods
//       .addAsset(multiplier1)
//       .accounts({
//         payer: payer.publicKey,
//         allassets: allAssets,
//         vaultAuthority: vaultAuthority,
//         mintAsset: mintAsset1.publicKey,
//         vaultAsset: vaultAsset1,
//         orderbook: PublicKey.findProgramAddressSync(
//           [Buffer.from("orderbook"), mintAsset1.publicKey.toBuffer()],
//           program.programId
//         )[0],
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     // --- Add the Second Asset ---
//     const multiplier2 = new BN(20);
//     await program.methods
//       .addAsset(multiplier2)
//       .accounts({
//         payer: payer.publicKey,
//         allassets: allAssets,
//         vaultAuthority: vaultAuthority,
//         mintAsset: mintAsset2.publicKey,
//         vaultAsset: vaultAsset2,
//         orderbook: PublicKey.findProgramAddressSync(
//           [Buffer.from("orderbook"), mintAsset2.publicKey.toBuffer()],
//           program.programId
//         )[0],
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();
//   });

//   it("Deposits and withdraws an asset", async () => {
//     const depositAmount = new BN(100 * 10 ** 6); // 100 tokens with 6 decimals

//     // Mint some tokens to the user so they can deposit
//     await mintTo(
//         provider.connection,
//         payer.payer,
//         mintAsset1.publicKey,
//         userAsset1TokenAccount,
//         payer.payer,
//         depositAmount.toNumber() * 2 // Mint twice the amount needed
//     );

//     const initialUserBalance = (await getAccount(provider.connection, userAsset1TokenAccount)).amount;
//     const initialVaultBalance = (await getAccount(provider.connection, vaultAsset1)).amount;

//     console.log(`Initial user balance: ${initialUserBalance}`);
//     console.log(`Initial vault balance: ${initialVaultBalance}`);

//     // --- Deposit Asset 1 ---
//     await program.methods.deposit(depositAmount).accounts({
//         payer: payer.publicKey,
//         sourceAccount: userAsset1TokenAccount,
//         vaultAsset: vaultAsset1,
//         mintAsset: mintAsset1.publicKey,
//         vaultAuthority: vaultAuthority,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//     }).rpc();
    
//     console.log("\n--- After Deposit ---");
    
//     let userBalanceAfterDeposit = (await getAccount(provider.connection, userAsset1TokenAccount)).amount;
//     let vaultBalanceAfterDeposit = (await getAccount(provider.connection, vaultAsset1)).amount;

//     console.log(`User balance after deposit: ${userBalanceAfterDeposit}`);
//     console.log(`Vault balance after deposit: ${vaultBalanceAfterDeposit}`);
    
//     // Assertions for deposit
//     assert.strictEqual(userBalanceAfterDeposit.toString(), new BN(initialUserBalance).sub(depositAmount).toString(), "User balance should decrease after deposit");
//     assert.strictEqual(vaultBalanceAfterDeposit.toString(), new BN(initialVaultBalance).add(depositAmount).toString(), "Vault balance should increase after deposit");

//     // --- Withdraw Asset 1 ---
//     const withdrawAmount = new BN(50 * 10 ** 6); // Withdraw half of the deposited amount

//     await program.methods.withdraw(withdrawAmount).accounts({
//         payer: payer.publicKey,
//         destinationAccount: userAsset1TokenAccount,
//         vaultAsset: vaultAsset1,
//         mintAsset: mintAsset1.publicKey,
//         vaultAuthority: vaultAuthority,
//         tokenProgram: TOKEN_PROGRAM_ID,
//         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId,
//     }).rpc();

//     console.log("\n--- After Withdraw ---");

//     let userBalanceAfterWithdraw = (await getAccount(provider.connection, userAsset1TokenAccount)).amount;
//     let vaultBalanceAfterWithdraw = (await getAccount(provider.connection, vaultAsset1)).amount;
    
//     console.log(`User balance after withdraw: ${userBalanceAfterWithdraw}`);
//     console.log(`Vault balance after withdraw: ${vaultBalanceAfterWithdraw}`);

//     // Assertions for withdraw
//     assert.strictEqual(userBalanceAfterWithdraw.toString(), new BN(userBalanceAfterDeposit).add(withdrawAmount).toString(), "User balance should increase after withdraw");
//     assert.strictEqual(vaultBalanceAfterWithdraw.toString(), new BN(vaultBalanceAfterDeposit).sub(withdrawAmount).toString(), "Vault balance should decrease after withdraw");

//   });
// });