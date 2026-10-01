import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PayfreightEscrow } from "../target/types/payfreight_escrow";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";

describe("payfreight_escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PayfreightEscrow as Program<PayfreightEscrow>;

  // System participants
  const shipper = anchor.web3.Keypair.generate();
  const carrier = anchor.web3.Keypair.generate();
  const admin = provider.wallet; // Current deployer acts as Admin

  // Token mints
  let usdcMint: anchor.web3.PublicKey;
  let payfreightMint: anchor.web3.PublicKey;

  // Token accounts
  let shipperUsdcAccount: anchor.web3.PublicKey;
  let shipperPayfreightAccount: anchor.web3.PublicKey;
  let carrierUsdcAccount: anchor.web3.PublicKey;
  let adminFeeUsdcAccount: anchor.web3.PublicKey;

  // Order parameters
  const orderId = "ORD-2026-001";
  const usdcAmount = new anchor.BN(1_000_000_000); // 1,000 USDC (6 decimals)
  const payfreightAmount = new anchor.BN(50_000_000); // 50 $PAYFREIGHT

  // PDA addresses (Escrow and Vaults)
  let escrowAccountPda: anchor.web3.PublicKey;
  let usdcVaultPda: anchor.web3.PublicKey;
  let payfreightVaultPda: anchor.web3.PublicKey;

  before(async () => {
    // 1. Airdrop SOL to participants for transaction fees
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(shipper.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(carrier.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    // 2. Create mock USDC and PAYFREIGHT mints for testing
    usdcMint = await createMint(
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
    shipperUsdcAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      usdcMint,
      shipper.publicKey
    );
    shipperPayfreightAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      payfreightMint,
      shipper.publicKey
    );
    carrierUsdcAccount = await createAssociatedTokenAccount(
      provider.connection,
      carrier,
      usdcMint,
      carrier.publicKey
    );
    adminFeeUsdcAccount = await createAssociatedTokenAccount(
      provider.connection,
      shipper,
      usdcMint,
      admin.publicKey
    );

    // 4. Mint test tokens to the shipper's account
    await mintTo(provider.connection, shipper, usdcMint, shipperUsdcAccount, admin.payer, 2_000_000_000);
    await mintTo(provider.connection, shipper, payfreightMint, shipperPayfreightAccount, admin.payer, 100_000_000);

    // 5. Derive PDA addresses
    [escrowAccountPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), Buffer.from(orderId)],
      program.programId
    );
    [usdcVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("usdc_vault"), Buffer.from(orderId)],
      program.programId
    );
    [payfreightVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("payfreight_vault"), Buffer.from(orderId)],
      program.programId
    );
  });

  it("1. Shipper successfully initializes the Escrow (initialize_escrow)", async () => {
    await program.methods
      .initializeEscrow(orderId, usdcAmount, payfreightAmount)
      .accounts({
        shipper: shipper.publicKey,
        carrier: carrier.publicKey,
        admin: admin.publicKey,
        escrowAccount: escrowAccountPda,
        usdcVault: usdcVaultPda,
        payfreightVault: payfreightVaultPda,
        shipperUsdcAccount: shipperUsdcAccount,
        shipperPayfreightAccount: shipperPayfreightAccount,
        usdcMint: usdcMint,
        payfreightMint: payfreightMint,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([shipper])
      .rpc();

    const escrowData = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.equal(escrowData.orderId, orderId);
    assert.equal(escrowData.isCmrUploaded, false);
    assert.equal(escrowData.isCompleted, false);
  });

  it("2. Carrier uploads the CMR document (upload_cmr)", async () => {
    const cmrHash = "ipfs://QmXyZ123456789CmrDocumentHash";

    await program.methods
      .uploadCmr(cmrHash)
      .accounts({
        carrier: carrier.publicKey,
        escrowAccount: escrowAccountPda,
      })
      .signers([carrier])
      .rpc();

    const escrowData = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.equal(escrowData.cmrHash, cmrHash);
    assert.equal(escrowData.isCmrUploaded, true);
  });

  it("3. Admin verifies CMR and releases funds (approve_and_release)", async () => {
    await program.methods
      .approveAndRelease()
      .accounts({
        admin: admin.publicKey,
        escrowAccount: escrowAccountPda,
        usdcVault: usdcVaultPda,
        payfreightVault: payfreightVaultPda,
        carrierUsdcAccount: carrierUsdcAccount,
        adminFeeUsdcAccount: adminFeeUsdcAccount,
        shipperPayfreightAccount: shipperPayfreightAccount,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    const escrowData = await program.account.escrowAccount.fetch(escrowAccountPda);
    assert.equal(escrowData.isCompleted, true);

    // Verify carrier balance (should receive 99.5% of 1000 USDC = 995 USDC)
    const carrierTokenAccount = await provider.connection.getTokenAccountBalance(carrierUsdcAccount);
    assert.equal(carrierTokenAccount.value.amount, "995000000");

    // Verify admin platform fee (0.5% of 1000 USDC = 5 USDC)
    const adminTokenAccount = await provider.connection.getTokenAccountBalance(adminFeeUsdcAccount);
    assert.equal(adminTokenAccount.value.amount, "5000000");
  });
});
