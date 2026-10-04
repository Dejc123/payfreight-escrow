<!--
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * This document is proprietary and confidential.
-->

# 🚚 PayFreight Escrow Smart Contract

> Trustless, secure, and automated freight escrow on Solana built with Anchor. PayFreight bridges the logistics industry with Web3 by automating payments, eliminating invoice delays, and introducing token-backed fee discounts.

---

## 📌 Overview & Hackathon Context

In traditional logistics, payment cycles for freight carriers often take 30 to 90 days, creating severe cash flow bottlenecks. Shippers also face risks of non-delivery or lost cargo documents (CMR).

PayFreight Escrow solves this by locking shipper funds into a secure on-chain escrow account on Solana. Funds are released automatically only after the transport is physically completed, verified via digital CMR documents, and approved by the platform administrator. Furthermore, shippers who stake or lock native `$PAYFREIGHT` tokens benefit from heavily discounted platform fees.

---

## 🔄 Business Logic & Workflow

The entire lifecycle of a shipment order flows through 3 main steps on the smart contract:

### 1. Escrow Initialization (`initialize_escrow`)
* The shipper creates an order and simultaneously deposits/locks the freight payment in **USDC** into a dedicated PDA vault.
* The shipper also locks an optimized amount of **1,000 `$PAYFREIGHT` tokens** into a token vault, acting as collateral and unlocking lower platform fees.

### 2. CMR Upload (`upload_cmr`)
* Once the carrier successfully delivers the cargo, they upload the cryptographic hash (or verified document URL) of the signed **CMR transport document** directly into the system.

### 3. Approval & Release (`approve_and_release`)
* The administrator reviews the uploaded CMR. Upon validation, the admin triggers the release function.
* The smart contract autonomously calculates a **dynamic platform fee** (0.5% if both parties stake, 1.0% if one stakes, or 1.5% standard fee otherwise).
* Net USDC is transferred to the carrier, the platform fee goes to the treasury, and the locked `$PAYFREIGHT` tokens are automatically returned to the shipper.

---

## 🏗️ Technical Architecture (Solana / Anchor)

The smart contract is written in **Rust** using the **Anchor Framework**. It utilizes Program Derived Addresses (PDAs) to handle secure token vaults without custodial risk.

* **Program ID:** `PayfrE1111111111111111111111111111111111111`
* **Key Components:**
  * `EscrowAccount`: Stores order metadata, participant public keys (shipper, carrier, admin), amounts, and status flags.
  * **USDC Vault (PDA):** Secures the freight funds during transit.
  * **PayFreight Vault (PDA):** Secures the native utility tokens locked by the shipper.

---

## 🛡️ Security & Access Control

* **State Validation:** Checks prevent double-spending or executing already completed orders (`AlreadyCompleted`).
* **Role Enforcement:** Strict authorization checks (`Signer` verification) ensure that only the designated carrier can upload the CMR and only the authorized admin can trigger the payout.
* **Autonomous Payouts:** Funds are safely transferred via CPI (Cross-Program Invocation) using PDA signer seeds.

---

## 🚀 Getting Started & Testing

### Prerequisites
Make sure you have the following installed:
* Solana CLI
* Anchor Framework
* Rust & Cargo

### Build & Test
```bash
# Clone the repository
git clone [https://github.com/Dejc123/payfreight-escrow.git](https://github.com/Dejc123/payfreight-escrow.git)
cd payfreight-escrow

# Build the Anchor program
anchor build

# Run tests
anchor test
