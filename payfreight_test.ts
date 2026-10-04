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
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PayfreightEscrowEur as Program<PayfreightEscrowEur>;

  // System participants
  const shipper = anchor.web3.Keypair.generate();
  const carrier = anchor.web3.Keypair.generate();
  const admin = provider.wallet; // Current deployer acts as Admin

  // Token mints
  let eurcMint: anchor.web3.PublicKey;
  let payfreightMint: anchor.web3.PublicKey;

  // Token accounts
  let shipperEurcAccount: anchor.web3.PublicKey;
  let shipperPayfreightAccount: anchor.web3.PublicKey;
  let carrierEurcAccount: anchor.web3.PublicKey;
  let adminFeeEurcAccount: anchor.web3.PublicKey;

  // Order parameters
  const orderId = "ORD-2026-EUR-001";
  const eurcAmount = new anchor.BN(1_000_000_000); // 1,000 EURC (6 decimals)
  const payfreightAmount = new anchor.BN(50_000_000); // 50 $PAYFREIGHT

  // PDA addresses (Escrow and Vaults)
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

    // 2. Create mock EURC and PAYFREIGHT mints for testing
    eurcMint = await createMint(
      provider.connection,
      shipper,
      admin.publicKey,
      null,
      6
    );
    payfreightMint = await createMint(
      provider.connection,
      shipper,
      admin.publicKey,
      null,
      6
    );

    // 3. Create associated token accounts for participants
    shipperEurcAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      eurcMint,
      shipper.publicKey
    );
    shipperPayfreightAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      payfreightMint,
      shipper.publicKey
    );
    carrierEurcAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      eurcMint,
      carrier.publicKey
    );
    adminFeeEurcAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      eurcMint,
      admin.publicKey
    );

    // 4. Mint test tokens to the shipper's account
    await mintTo(provider.connection, shipper, eurcMint, shipperEurcAccount, admin.payer, 2_000_000_000);
    await mintTo(provider.connection, shipper, payfreightMint, shipperPayfreightAccount, admin.payer, 100_000_000);

    // 5. Derive PDA addresses matching the smart contract seeds
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

  it("1. Shipper successfully initializes the EUR Escrow (initialize_escrow_eur)", async () => {
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

  it("2. Carrier uploads the CMR document (upload_cmr_eur)", async () => {
    const cmrHash = "ipfs://QmXyZ123456789CmrDocumentHashEur";

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

  it("3. Admin verifies CMR and releases funds (approve_and_release_eur)", async () => {
    await program.methods
      .approveAndReleaseEur()
      .accounts({
        admin: admin.publicKey,
        escrowAccount: escrowAccountPda,
        eurcVault: eurcVaultPda,
        payfreightVault: payfreightVaultPda,
        carrierEurcAccount: carrierEurcAccount,
        adminFeeEurcAccount: adminFeeEurcAccount,
        shipperPayfreightAccount: shipperPayfreightAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    })
    .rpc();

    const escrowData = await program.account.escrowAccountEur.fetch(escrowAccountPda);
    assert.equal(escrowData.isCompleted, true);

    // Verify carrier balance (receives 99.5% of 1,000 EURC = 995 EURC due to staking discount)
    const carrierTokenAccount = await provider.connection.getTokenAccountBalance(carrierEurcAccount);
    assert.equal(carrierTokenAccount.value.amount, "995000000");

    // Verify admin platform fee (0.5% of 1,000 EURC = 5 EURC)
    const adminTokenAccount = await provider.connection.getTokenAccountBalance(adminFeeEurcAccount);
    assert.equal(adminTokenAccount.value.amount, "5000000");
  });
});
