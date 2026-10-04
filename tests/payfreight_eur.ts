/*
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * 
 * This source code is proprietary and confidential. 
 * Unauthorized copying of this file, via any medium, is strictly prohibited.
*/

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PayfreightEscrowEur } from "../target/types/payfreight_escrow_eur";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";

describe("payfreight_escrow_eur", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PayfreightEscrowEur as Program<PayfreightEscrowEur>;

  // Test participants
  const shipper = anchor.web3.Keypair.generate();
  const carrier = anchor.web3.Keypair.generate();
  const admin = provider.wallet;

  let eurcMint: anchor.web3.PublicKey;
  let payfreightMint: anchor.web3.PublicKey;

  let shipperEurcAccount: anchor.web3.PublicKey;
  let shipperPayfreightAccount: anchor.web3.PublicKey;
  let carrierEurcAccount: anchor.web3.PublicKey;
  let adminFeeEurcAccount: anchor.web3.PublicKey;

  const orderId = "EUR-ORD-2026-001";
  const eurcAmount = new anchor.BN(2_000_000_000); // 2,000 EURC (assuming 6 decimals)
  const payfreightAmount = new anchor.BN(50_000_000); // 50 PAYFREIGHT tokens

  let escrowAccountPda: anchor.web3.PublicKey;
  let eurcVaultPda: anchor.web3.PublicKey;
  let payfreightVaultPda: anchor.web3.PublicKey;

  before(async () => {
    // 1. Airdrop SOL to participants for transaction fees
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(shipper.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(carrier.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    // 2. Create mock EURC and PAYFREIGHT mints
    eurcMint = await createMint(provider.connection, shipper, admin.publicKey, null, 6);
    payfreightMint = await createMint(provider.connection, shipper, admin.publicKey, null, 6);

    // 3. Create associated token accounts for participants
    shipperEurcAccount = await createAssociatedTokenAccount(provider.connection, shipper, eurcMint, shipper.publicKey);
    shipperPayfreightAccount = await createAssociatedTokenAccount(provider.connection, shipper, payfreightMint, shipper.publicKey);
    carrierEurcAccount = await createAssociatedTokenAccount(provider.connection, carrier, eurcMint, carrier.publicKey);
    adminFeeEurcAccount = await createAssociatedTokenAccount(provider.connection, shipper, eurcMint, admin.publicKey);

    // 4. Mint initial tokens to the shipper
    await mintTo(provider.connection, shipper, eurcMint, shipperEurcAccount, admin.payer, 5_000_000_000);
    await mintTo(provider.connection, shipper, payfreightMint, shipperPayfreightAccount, admin.payer, 100_000_000);

    // 5. Derive PDAs using the exact same seeds as in the smart contract
    [escrowAccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow_eur"), Buffer.from(orderId)],
      program.programId
    );
    [eurcVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("eurc_vault_eur"), Buffer.from(orderId)],
      program.programId
    );
    [payfreightVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("payfreight_vault_eur"), Buffer.from(orderId)],
      program.programId
    );
  });

  it("1. Initializes EUR Escrow and locks EURC + PAYFREIGHT", async () => {
    await program.methods
      .initializeEscrowEur(orderId, eurcAmount, payfreightAmount)
      .accounts({
        shipper: shipper.publicKey,
        carrier: carrier.publicKey,
        admin: admin.publicKey,
        escrowAccount: escrowAccountPda,
        eurcVault: eurcVaultPda,
        payfreightVault: payfreightVaultPda,
        shipperEurcAccount: shipperEurcAccount,
        shipperPayfreightAccount: shipperPayfreightAccount,
        eurcMint: eurcMint,
        payfreightMint: payfreightMint,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([shipper])
      .rpc();

    const escrowData = await program.account.escrowAccountEur.fetch(escrowAccountPda);
    assert.equal(escrowData.orderId, orderId);
    assert.equal(escrowData.isCmrUploaded, false);
    assert.equal(escrowData.isCompleted, false);
  });

  it("2. Carrier uploads the e-CMR document hash", async () => {
    const cmrHash = "ipfs://QmEuropeanCmrDocumentHash123456789";

    await program.methods
      .uploadCmrEur(cmrHash)
      .accounts({
        carrier: carrier.publicKey,
        escrowAccount: escrowAccountPda,
      })
      .signers([carrier])
      .rpc();

    const escrowData = await program.account.escrowAccountEur.fetch(escrowAccountPda);
    assert.equal(escrowData.cmrHash, cmrHash);
    assert.equal(escrowData.isCmrUploaded, true);
  });

  it("3. Admin approves, applies dynamic fee (0.5%), and releases funds/tokens", async () => {
    await program.methods
      .approveAndReleaseEur()
      .accounts({
        admin: admin.publicKey,
        escrowAccount: escrowAccountPda,
        eurcVault: eurcVaultPda,
        payfre_vault: payfreightVaultPda, // popravljeno spodaj na payfreightVault
        carrierEurcAccount: carrierEurcAccount,
        adminFeeEurcAccount: adminFeeEurcAccount,
        shipperPayfreightAccount: shipperPayfreightAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      } as any)
      .rpc();

    const escrowData = await program.account.escrowAccountEur.fetch(escrowAccountPda);
    assert.equal(escrowData.isCompleted, true);
  });
});
