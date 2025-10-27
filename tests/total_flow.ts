import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SmartContract } from "../target/types/smart_contract";
import {
  getAssociatedTokenAddressSync,
  createMint,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";
import { PublicKey } from "@solana/web3.js";

describe("test place bid", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SmartContract as Program<SmartContract>;
  const payer = provider.wallet as anchor.Wallet;

  // Keypairs and PDAs will be stored here
  let mintAsset1: anchor.web3.PublicKey;
  let mintAsset2: anchor.web3.PublicKey;
  
  let baseAsset: anchor.web3.PublicKey; // Base asset for the market we create in this test
  let allAssetsPda: anchor.web3.PublicKey;
  let configPda: anchor.web3.PublicKey;
  let vaultAuthorityPda: anchor.web3.PublicKey;

  // PDAs for Asset 1
  let vaultAssetPda1: anchor.web3.PublicKey;

  // PDAs for Asset 2
  let vaultAssetPda2: anchor.web3.PublicKey;

  // Set initial market parameters
  const startTick = new anchor.BN(100); // e.g., represents 1.00%
  const tickSize = new anchor.BN(10); // e.g., represents 0.10%

  let sourceAccount: anchor.web3.PublicKey;

  it("Is initialized!", async () => {
    baseAsset = await createMint(
      provider.connection,
      payer.payer,
      payer.publicKey,
      null,
      1 // Decimals
    );
    console.log("Created Base Asset Mint:", baseAsset.toBase58());

    // Derive PDAs for initialization
    [allAssetsPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("all_assets"), baseAsset.toBuffer()],
      program.programId
    );

    // Call the initialize instruction
    const tx = await program.methods
      .initialize(startTick, tickSize)
      .accounts({
        payer: payer.publicKey,
        baseAsset: baseAsset,
        allAssets: allAssetsPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize transaction signature", tx);

    // Verify that the state was set correctly
    const allAssetsAccount = await program.account.allAssets.fetch(allAssetsPda);
    assert.ok(allAssetsAccount.startTick.eq(startTick), "Start tick should match");
    assert.ok(allAssetsAccount.tickSize.eq(tickSize), "Tick size should match");
    assert.equal(allAssetsAccount.sizeAssets, 0, "Initial last index should be 0");
  });

  it("Adds the first asset", async () => {
    // Create a new mint for the first asset
    mintAsset1 = await createMint(
      provider.connection,
      payer.payer,
      payer.publicKey,
      null,
      6 // Decimals
    );
    console.log("Created Mint for Asset 1:", mintAsset1.toBase58());

    // Derive PDAs needed for adding an asset
    [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("vault_authority")],
        program.programId
    );
    
    // Get the associated token address for the vault
    vaultAssetPda1 = getAssociatedTokenAddressSync(mintAsset1, vaultAuthorityPda, true);

    const leverage = new anchor.BN(2 * 1000); // Example leverage = SCALE_LEVERAGE (2x)

    // Call the add_asset instruction
    const tx = await program.methods
      .addAsset(leverage)
      .accounts({
        payer: payer.publicKey,
        allAssets: allAssetsPda,
        vaultAuthority: vaultAuthorityPda,
        mintAsset: mintAsset1,
        vaultAsset: vaultAssetPda1,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Add Asset 1 transaction signature", tx);

    // Verify state
    const allAssetsAccount = await program.account.allAssets.fetch(allAssetsPda);
    assert.equal(allAssetsAccount.sizeAssets, 1, "Last index should be 1");
    const assetInfo = allAssetsAccount.assets[0];
    assert.ok(assetInfo.mint.equals(mintAsset1), "Mint public key should match");
    assert.ok(assetInfo.leverage.eq(leverage), "Leverage should match");
  });

  it("Adds the second asset", async () => {
    // Create a new mint for the second asset
    mintAsset2 = await createMint(
      provider.connection,
      payer.payer,
      payer.publicKey,
      null,
      9 // Decimals
    );
    console.log("Created Mint for Asset 2:", mintAsset2.toBase58());
    
    vaultAssetPda2 = getAssociatedTokenAddressSync(mintAsset2, vaultAuthorityPda, true);

    const leverage = new anchor.BN(3 * 1000); // Example leverage = SCALE_LEVERAGE (3x)

    // Call the add_asset instruction
    const tx = await program.methods
      .addAsset(leverage)
      .accounts({
        payer: payer.publicKey,
        allAssets: allAssetsPda,
        vaultAuthority: vaultAuthorityPda,
        mintAsset: mintAsset2,
        vaultAsset: vaultAssetPda2,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    
    console.log("Add Asset 2 transaction signature", tx);

    // Verify state
    const allAssetsAccount = await program.account.allAssets.fetch(allAssetsPda);
    assert.equal(allAssetsAccount.sizeAssets, 2, "Last index should be 2 after adding second asset");
    const assetInfo = allAssetsAccount.assets[1];
    assert.ok(assetInfo.mint.equals(mintAsset2), "Mint public key for asset 2 should match");
  });

  it("Deposits and withdraws an asset", async () => {
    const depositAmount = new anchor.BN(100 * 10 ** 6); // 100 tokens with 6 decimals

    // Create a token account for the payer and mint some tokens to it
    sourceAccount = await createAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mintAsset1,
      payer.publicKey
    );
    const mintAmount = new anchor.BN(1000 * 10 ** 6); // 1000 tokens with 6 decimals
    await mintTo(
      provider.connection,
      payer.payer,
      mintAsset1,
      sourceAccount,
      payer.payer,
      BigInt(mintAmount.toString())
    );
    console.log(`Minted ${mintAmount} tokens of Asset 1 to payer's account.`);

    // Mint to me also for the web ui
    let sourceAccount2 = await createAssociatedTokenAccount(
      provider.connection,
      payer.payer,
      mintAsset1,
      new PublicKey("3vViNA8Cw3gzZySscrYMnHzUMB1ZwqP8MqifGVX2bEin")
    );
    await mintTo(
      provider.connection,
      payer.payer,
      mintAsset1,
      sourceAccount2,
      payer.payer,
      BigInt(mintAmount.toString())
    );
    console.log(`Minted ${mintAmount} tokens of Asset 1 to my acc.`);

    // PDA for the deposit
    const [lenderDepositPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("lender_deposit"),
        payer.publicKey.toBuffer(),
      ],
      program.programId
    );

    // Ensure the user has enough tokens to deposit (they were minted in the previous test)
    const initialUserBalance = (await getAccount(provider.connection, sourceAccount)).amount;
    const initialVaultBalance = (await getAccount(provider.connection, vaultAssetPda1)).amount;

    console.log(`Initial user balance: ${initialUserBalance}`);
    console.log(`Initial vault balance: ${initialVaultBalance}`);
    
    await program.methods
      .deposit(depositAmount) // Argument is now an array
      .accounts({
          // Accounts from the DepositMultiple struct
          payer: payer.publicKey,
          allAssets: allAssetsPda,
          lenderDeposit: lenderDepositPda,
          vaultAuthority: vaultAuthorityPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
      })
      .remainingAccounts([
          // Accounts are passed here in order: [source, vault, mint]
          { pubkey: sourceAccount, isSigner: false, isWritable: true },
          { pubkey: vaultAssetPda1, isSigner: false, isWritable: true },
          { pubkey: mintAsset1, isSigner: false, isWritable: false },
      ])
      .rpc();
    console.log("\n--- After Deposit ---");

    let userBalanceAfterDeposit = (await getAccount(provider.connection, sourceAccount)).amount;
    let vaultBalanceAfterDeposit = (await getAccount(provider.connection, vaultAssetPda1)).amount;

    console.log(`User balance after deposit: ${userBalanceAfterDeposit}`);
    console.log(`Vault balance after deposit: ${vaultBalanceAfterDeposit}`);
    
    // Assertions for deposit
    // Todo, once the code is all_assets.rs, re enable these assertions
    // assert.strictEqual(userBalanceAfterDeposit.toString(), new anchor.BN(initialUserBalance.toString()).sub(depositAmount).toString(), "User balance should decrease after deposit");
    // assert.strictEqual(vaultBalanceAfterDeposit.toString(), new anchor.BN(initialVaultBalance.toString()).add(depositAmount).toString(), "Vault balance should increase after deposit");

    // --- Withdraw Asset 1 ---
    const withdrawAmount = new anchor.BN(50 * 10 ** 6); // Half of the deposited amount

    await program.methods
      .withdraw(withdrawAmount) // The total amount to withdraw
      .accounts({
          // Accounts from the Withdraw struct
          payer: payer.publicKey,
          allAssets: allAssetsPda,
          lenderDeposit: lenderDepositPda,
          vaultAuthority: vaultAuthorityPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
      })
      .remainingAccounts([
          // Accounts are passed here in order: [destination, vault, mint]
          { pubkey: sourceAccount, isSigner: false, isWritable: true }, // destination account = sourceAccount
          { pubkey: vaultAssetPda1, isSigner: false, isWritable: true },
          { pubkey: mintAsset1, isSigner: false, isWritable: false },
      ])
      .rpc();

    console.log("\n--- After Withdraw ---");

    let userBalanceAfterWithdraw = (await getAccount(provider.connection, sourceAccount)).amount;
    let vaultBalanceAfterWithdraw = (await getAccount(provider.connection, vaultAssetPda1)).amount;
    
    console.log(`User balance after withdraw: ${userBalanceAfterWithdraw}`);
    console.log(`Vault balance after withdraw: ${vaultBalanceAfterWithdraw}`);

    // Assertions for withdraw
    // Todo, once the code is all_assets.rs, re enable these assertions
    // assert.strictEqual(userBalanceAfterWithdraw.toString(), new anchor.BN(userBalanceAfterDeposit.toString()).add(withdrawAmount).toString(), "User balance should increase after withdraw");
    // assert.strictEqual(vaultBalanceAfterWithdraw.toString(), new anchor.BN(vaultBalanceAfterDeposit.toString()).sub(withdrawAmount).toString(), "Vault balance should decrease after withdraw");
  });

  it("Places a bid for the first asset", async () => {

    // Define bid parameters
    const assetIndex = new anchor.BN(0); // First asset added
    const slotIndex = new anchor.BN(0); // e.g., represents 1.10%
    const bidAmount = new anchor.BN(100 * 10 ** 6); // Bid 100 tokens

    // Derive PDA for the lender's deposit "ticket"
    const [looperDepositPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("looper_deposit"),
        payer.publicKey.toBuffer(),
        assetIndex.toBuffer("le", 8), // asset must be little-endian 8 bytes
        slotIndex.toBuffer("le", 8), // tick must be little-endian 8 bytes
      ],
      program.programId
    );

    // // Derive PDAs for initialization
    // [allAssetsPda] = anchor.web3.PublicKey.findProgramAddressSync(
    //   [Buffer.from("all_assets"), baseAsset.toBuffer()],
    //   program.programId
    // );

    let allAssetsData = await program.account.allAssets.fetch(allAssetsPda);
    console.log("Orderbook Account before the asset deposited:", allAssetsData.assets[assetIndex.toNumber()].orderbook.slots);

    // Call the place_bid instruction
    const tx = await program.methods
      .placeBid(assetIndex, slotIndex, bidAmount)
      .accounts({
        payer: payer.publicKey,
        allAssets: allAssetsPda,
        vaultAuthority: vaultAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Place Bid transaction signature", tx);

    allAssetsData = await program.account.allAssets.fetch(allAssetsPda);
    console.log("Orderbook Account after the asset deposited:", allAssetsData.assets[assetIndex.toNumber()].orderbook.slots);

    // Verify the state of the orderbook and lender deposit
    allAssetsData = await program.account.allAssets.fetch(allAssetsPda);

    assert.ok(allAssetsData.assets[assetIndex.toNumber()].orderbook.slots[slotIndex.toNumber()].eq(bidAmount), "Orderbook slot should be updated with the bid amount");

    const looperDepositAccount = await program.account.looperDeposit.fetch(looperDepositPda);
    assert.ok(looperDepositAccount.looper.equals(payer.publicKey), "Lender should be the payer");
    assert.ok(looperDepositAccount.amount.eq(bidAmount), "Lender deposit amount should match bid amount");
    assert.ok(looperDepositAccount.slotIndex.eq(slotIndex), "Lender deposit slot index should match bid slot index");
    
    
  });

});