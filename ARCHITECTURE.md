<!--
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * This document is proprietary and confidential.
-->

# PayFreight Escrow - Architecture & Financial Flow

## Overview
PayFreight Escrow is a decentralized logistics finance protocol built on Solana using the Anchor framework. It bridges real-world transport regulations with blockchain-backed escrow and deflationary tokenomics.

## Financial & Escrow Flow

1. **Initialization (`initialize_escrow`):**
   - **Shipper** locks funds into dedicated Program Derived Addresses (PDAs):
     - **EURC/Stablecoin Vault:** Transport Fare + Fixed Cancellation Fee.
     - **PAYF Vault:** Optional PAYF staking tokens for credibility and fee discounts.

2. **Execution & e-CMR (`upload_cmr`):**
   - The **Carrier** uploads the signed electronic CMR hash upon delivery.

3. **Resolution & Payout (`resolve_deal`):**
   - **Scenario A (Normal Delivery):** Carrier receives the transport fare. Shipper is fully refunded the cancellation fee.
   - **Scenario B (Carrier Cancellation / No-Show):** Shipper receives a full refund + penalty from the carrier.

## Tokenomics & Deflationary Burn
- **Optional Staking:** Participants can lock **1,000 PAYF tokens** (subject to future halving adjustments) to filter out bad actors and reduce platform fees.
- **Fee Tiers based on Staking:**
  - **Both parties stake:** Lowest platform fee (0.5%)
  - **One party stakes:** Standard platform fee (1.0%)
  - **Neither stakes:** Base platform fee (1.5%)
- **Burn Mechanism:** Transaction fees trigger a dynamic deflationary burn (starting at 1 PAYF, scaling down to 0.5 and 0.25 PAYF per transaction over time to support mass adoption).
