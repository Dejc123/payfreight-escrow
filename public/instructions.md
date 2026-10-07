# PayFreight Protocol – Official Instructions & Compliance Guide (2026 Edition)

## 1. Introduction & Scope
The PayFreight Protocol is a global, bank-grade settlement and institutional escrow engine built for United States (ACH/FedNow) and European (SEPA Instant) freight corridors. It integrates ISO 20022 messaging standards and UN/CEFACT e-CMR protocols to eliminate counterparty risk and payment defaults.

## 2. Core Operational Rules (The 5-Point Safeguard)
1. **Non-Custodial Smart Vaults:** All funds are locked in 1:1 fiat-backed stability (USD/EUR). PayFreight operators hold zero manual authority over locked escrow balances.
2. **Commitment Fee Architecture (€250 / $250):** Both Shipper and Carrier lock a mutual commitment deposit upon booking to eliminate ghosting and last-minute disruptions on loading day.
3. **Immediate Principal Liquidity Protection:** In the event of a transport cancellation, the main freight principal is **instantly returned to the Shipper**, ensuring capital is never trapped during disputes, allowing the shipper to hire a replacement carrier immediately.
4. **Automated e-CMR / POD Verification:** Release of final payout is triggered programmatically via verified electronic consignment notes (e-CMR) or proof of delivery signatures.
5. **Multi-Language Regulatory Compliance:** Protocol documentation and notices are structured to support multi-jurisdictional standards across 25+ European and North American operational regions.

## 3. Dispute Resolution & Fault Attribution Workflow
* **Mutual Cancellation:** If both parties agree via email token confirmation before dispatch, 100% of freight principal and both €250/$250 deposits are refunded with zero fees.
* **Unilateral Fault Attribution:** When a party triggers the cancellation form due to loading-day failure, the counterpart receives a tokenized notification. Unresponsiveness exceeding 24 hours triggers automated protocol resolution.
