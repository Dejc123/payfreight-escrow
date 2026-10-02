<!--
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: Dejc123 & Team
 * This document is proprietary and confidential.
-->

# PayFreight Escrow - Architecture & Financial Flow

## Overview
PayFreight Escrow is a decentralized logistics finance protocol built on Solana using the Anchor framework. It bridges real-world transport regulations (CMR, demurrage, cancellations) with blockchain-backed escrow and deflationary tokenomics.

## Financial & Escrow Flow (EUR Version)

1. **Initialization (`initialize_escrow_eur`):**
   - **Shipper** locks funds into dedicated Program Derived Addresses (PDAs):
     - **EURC Vault:** Transport Fare + Fixed Cancellation Fee (€250) + Max Waiting Fee (€600 for 3 days @ €200/day).
     - **PAYF Vault:** PAYFREIGHT tokens locked as a staking/credibility deposit.
   
2. **Execution & e-CMR (`upload_cmr_eur`):**
   - The **Carrier** uploads the signed electronic CMR hash upon loading/delivery.

3. **Resolution & Payout (`resolve_deal_eur`):**
   - The **Admin/Mediator** verifies the CMR data and resolves the deal based on 3 distinct scenarios:
     - **Scenario A (Normal Delivery):** Carrier receives the transport fare. Shipper is fully refunded the unused waiting pool (€600) and cancellation fee (€250).
     - **Scenario B (Waiting Days / Delays):** Carrier receives transport fare + daily waiting fees (€200/day, up to 3 days). Unused waiting days and the cancellation fee are returned to the shipper.
     - **Scenario C (Carrier No-Show / Cancellation):** Shipper receives a full refund of their deposit + an extra €250 penalty from the carrier. The carrier’s account is debited by €250.

## Tokenomics & Deflationary Burn
- **Staking:** Both parties lock PAYFREIGHT tokens to participate, filtering out bad actors.
- **Burn Mechanism:** Exactly 10 PAYFREIGHT tokens are permanently burned (`token::burn`) upon the resolution of every deal.
- **Dynamic Demand Scaling (Bitcoin-Style Halving Model):**
  - To prevent long-term price appreciation from pricing out carriers as platform demand scales, required staking amounts and burn fees are structured to decrease periodically (similar to Bitcoin's 4-year halving cycle):
    - **Years 0–4:** 10,000 PAYF stake / 10 PAYF burn
    - **Years 4–8:** 5,000 PAYF stake / 5 PAYF burn
    - **Years 8–12:** 2,500 PAYF stake / 2.5 PAYF burn
