# Payfreight Escrow Smart Contract (Solana / Anchor)

Secure escrow smart contract for transport logistics, built on the Solana blockchain using the Anchor framework.

---

## 🚀 Core Mechanics & Business Logic

This document describes the 100% core mechanics of the Payfreight system. All other features are secondary; this core engine solves a real-world logistics problem while directly driving token utility and value.

### Transaction Lifecycle & Step-by-Step Flow

1. **Escrow Initialization (by Shipper)**
   * The shipper creates a transport order.
   * The smart contract requires depositing the following into a unique, dedicated Escrow account in a single transaction:
     * **Freight Payment** (e.g., 1,000 USDC)
     * **Required $PAYFREIGHT Tokens** (acting as collateral and a mandatory condition for lower platform fees).

2. **Fulfillment & CMR Upload (by Carrier)**
   * The carrier picks up the cargo, executes the transport, and obtains a signed and stamped CMR delivery note at the unloading destination.
   * The carrier uploads the scanned or photographed proof (verified CMR) into the Payfreight interface.

3. **Manual Admin Approval**
   * You or your operations team open the uploaded CMR on the admin dashboard.
   * Verify that the document is legible, signed by the receiver, and matches the order parameters.
   * Click **Approve**.

4. **Automated Execution**
   * Your approval triggers an instruction to the Solana smart contract.
   * In a single atomic transaction, the smart contract executes three actions simultaneously:
     * **Carrier Payout:** Transfers the net freight amount in USDC to the carrier (e.g., 995 USDC).
     * **Platform Fee:** Transfers the platform fee in USDC to the Payfreight wallet (e.g., 5 USDC).
     * **Token Return:** Unlocks and returns the locked $PAYFREIGHT tokens back to the shipper.

### Why This Core Engine is Exceptional

* **For Shippers:** Shippers must buy and hold $PAYFREIGHT tokens to create orders with reduced fees, establishing an organic, mandatory entry barrier and utility for the token.
* **For Carriers:** Complete payment security. Funds are locked upfront (eliminating default risk), and payout is guaranteed immediately upon CMR verification.
* **For Cash Flow:** Instantaneous USDC fee generation with every approved delivery.
* **For Token Value:** As logistics volume scales, more $PAYFREIGHT tokens are constantly locked in active escrows, decreasing circulating supply on the open market and driving upward price pressure.

---

## 🛠️ How to Build & Test

Prerequisites:
- Rust v1.75+
- Solana CLI
- Anchor Framework

### Run Tests:
```bash
yarn install
anchor test
