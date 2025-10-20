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
  
  let allAssetsPda: anchor.web3.PublicKey;
  let configPda: anchor.web3.PublicKey;
  let vaultAuthorityPda: anchor.web3.PublicKey;

  // PDAs for Asset 1
  let orderbookPda1: anchor.web3.PublicKey;
  let vaultAssetPda1: anchor.web3.PublicKey;

  // PDAs for Asset 2
  let orderbookPda2: anchor.web3.PublicKey;
  let vaultAssetPda2: anchor.web3.PublicKey;

  // Set initial market parameters
  const startTick = new anchor.BN(100); // e.g., represents 1.00%
  const tickSize = new anchor.BN(10); // e.g., represents 0.10%

  let sourceAccount: anchor.web3.PublicKey;

  it("Is initialized!", async () => {
    // Derive PDAs for initialization
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
    [allAssetsPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("allassets")],
      program.programId
    );

    // Call the initialize instruction
    const tx = await program.methods
      .initialize(startTick, tickSize)
      .accounts({
        payer: payer.publicKey,
        config: configPda,
        allassets: allAssetsPda,
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
    [orderbookPda1] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("orderbook"), mintAsset1.toBuffer()],
        program.programId
    );

    // Get the associated token address for the vault
    vaultAssetPda1 = getAssociatedTokenAddressSync(mintAsset1, vaultAuthorityPda, true);

    const multiplier = new anchor.BN(95); // Example multiplier

    // Call the add_asset instruction
    const tx = await program.methods
      .addAsset(multiplier)
      .accounts({
        payer: payer.publicKey,
        allassets: allAssetsPda,
        vaultAuthority: vaultAuthorityPda,
        mintAsset: mintAsset1,
        vaultAsset: vaultAssetPda1,
        orderbook: orderbookPda1,
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
    assert.ok(assetInfo.vault.equals(vaultAssetPda1), "Vault public key should match");
    assert.ok(assetInfo.multiplier.eq(multiplier), "Multiplier should match");
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
    
    // Derive PDAs for the second asset
    [orderbookPda2] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("orderbook"), mintAsset2.toBuffer()],
        program.programId
    );
    vaultAssetPda2 = getAssociatedTokenAddressSync(mintAsset2, vaultAuthorityPda, true);

    const multiplier = new anchor.BN(90); // Example multiplier

    // Call the add_asset instruction
    const tx = await program.methods
      .addAsset(multiplier)
      .accounts({
        payer: payer.publicKey,
        allassets: allAssetsPda,
        vaultAuthority: vaultAuthorityPda,
        mintAsset: mintAsset2,
        vaultAsset: vaultAssetPda2,
        orderbook: orderbookPda2,
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

    

    // Ensure the user has enough tokens to deposit (they were minted in the previous test)
    const initialUserBalance = (await getAccount(provider.connection, sourceAccount)).amount;
    const initialVaultBalance = (await getAccount(provider.connection, vaultAssetPda1)).amount;

    console.log(`Initial user balance: ${initialUserBalance}`);
    console.log(`Initial vault balance: ${initialVaultBalance}`);

    // --- Deposit Asset 1 ---
    await program.methods.deposit(depositAmount).accounts({
        payer: payer.publicKey,
        sourceAccount: sourceAccount,
        vaultAsset: vaultAssetPda1,
        mintAsset: mintAsset1,
        vaultAuthority: vaultAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();
    
    console.log("\n--- After Deposit ---");

    let userBalanceAfterDeposit = (await getAccount(provider.connection, sourceAccount)).amount;
    let vaultBalanceAfterDeposit = (await getAccount(provider.connection, vaultAssetPda1)).amount;

    console.log(`User balance after deposit: ${userBalanceAfterDeposit}`);
    console.log(`Vault balance after deposit: ${vaultBalanceAfterDeposit}`);
    
    // Assertions for deposit
    assert.strictEqual(userBalanceAfterDeposit.toString(), new anchor.BN(initialUserBalance.toString()).sub(depositAmount).toString(), "User balance should decrease after deposit");
    assert.strictEqual(vaultBalanceAfterDeposit.toString(), new anchor.BN(initialVaultBalance.toString()).add(depositAmount).toString(), "Vault balance should increase after deposit");

    // --- Withdraw Asset 1 ---
    const withdrawAmount = new anchor.BN(50 * 10 ** 6); // Withdraw half of the deposited amount

    await program.methods.withdraw(withdrawAmount).accounts({
        payer: payer.publicKey,
        destinationAccount: sourceAccount,
        vaultAsset: vaultAssetPda1,
        mintAsset: mintAsset1,
        vaultAuthority: vaultAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    console.log("\n--- After Withdraw ---");

    let userBalanceAfterWithdraw = (await getAccount(provider.connection, sourceAccount)).amount;
    let vaultBalanceAfterWithdraw = (await getAccount(provider.connection, vaultAssetPda1)).amount;
    
    console.log(`User balance after withdraw: ${userBalanceAfterWithdraw}`);
    console.log(`Vault balance after withdraw: ${vaultBalanceAfterWithdraw}`);

    // Assertions for withdraw
    assert.strictEqual(userBalanceAfterWithdraw.toString(), new anchor.BN(userBalanceAfterDeposit.toString()).add(withdrawAmount).toString(), "User balance should increase after withdraw");
    assert.strictEqual(vaultBalanceAfterWithdraw.toString(), new anchor.BN(vaultBalanceAfterDeposit.toString()).sub(withdrawAmount).toString(), "Vault balance should decrease after withdraw");
  });

  it("Places a bid for the first asset", async () => {

    // Define bid parameters
    const bidIndex = new anchor.BN(0); // e.g., represents 1.10%
    const bidAmount = new anchor.BN(100 * 10 ** 6); // Bid 100 tokens

    // Derive PDA for the lender's deposit "ticket"
    const [lenderDepositPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("lender_deposit"),
        payer.publicKey.toBuffer(),
        mintAsset1.toBuffer(),
        bidIndex.toBuffer("le", 8), // tick must be little-endian 8 bytes
      ],
      program.programId
    );

    // Call the place_bid instruction
    const tx = await program.methods
      .placeBid(bidIndex, bidAmount)
      .accounts({
        payer: payer.publicKey,
        allassets: allAssetsPda,
        orderbook: orderbookPda1,
        mintAsset: mintAsset1,
        lenderDeposit: lenderDepositPda,
        sourceAccount: sourceAccount,
        vaultAsset: vaultAssetPda1,
        vaultAuthority: vaultAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Place Bid transaction signature", tx);

    // Verify the state of the orderbook and lender deposit
    const orderbookAccount = await program.account.orderbook.fetch(orderbookPda1);
    const slotIndex = bidIndex.toNumber();

    assert.ok(orderbookAccount.slots[slotIndex].eq(bidAmount), "Orderbook slot should be updated with the bid amount");

    const lenderDepositAccount = await program.account.lenderDeposit.fetch(lenderDepositPda);
    assert.ok(lenderDepositAccount.lender.equals(payer.publicKey), "Lender should be the payer");
    assert.ok(lenderDepositAccount.amount.eq(bidAmount), "Lender deposit amount should match bid amount");
    assert.ok(lenderDepositAccount.slotIndex.eq(bidIndex), "Lender deposit slot index should match bid slot index");
    console.log("Orderbook Account:", orderbookAccount);
  });

});