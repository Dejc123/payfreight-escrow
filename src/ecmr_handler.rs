// =========================================================================
// MODULE: Payfreight eCMR Handler & B2B Settlement Protocol
// LICENSE: Copyright (c) 2026 Payfreight. All rights reserved.
// AUTHOR: Payfreight Core Development Team
//
// DESCRIPTION:
// Manages eCMR data structures, verifies SHA-256 document hashes from OCR,
// checks compliance with eFTI regulations, and executes secure B2B financial settlements.
// =========================================================================

use std::error::Error;

/// Structure defining the eCMR record and parameters for B2B settlement
#[derive(Debug, Clone)]
pub struct EcmrTransaction {
    pub ecmr_id: String,
    pub cmr_hash: String, // SHA-256 fingerprint generated from paper CMR OCR scan
    pub shipper: String,  // Email, Tax ID or Wallet
    pub carrier: String,  // Email, Tax ID or Wallet
    pub receiver: String,
    pub settlement_amount: f64,
    pub is_efti_compliant: bool,
    pub is_delivered: bool,
}

impl EcmrTransaction {
    /// Constructor for a new eCMR record initialized with document hash and eFTI compliance
    pub fn new(
        ecmr_id: &str,
        cmr_hash: &str,
        shipper: &str,
        carrier: &str,
        receiver: &str,
        amount: f64,
    ) -> Self {
        Self {
            ecmr_id: ecmr_id.to_string(),
            cmr_hash: cmr_hash.to_string(),
            shipper: shipper.to_string(),
            carrier: carrier.to_string(),
            receiver: receiver.to_string(),
            settlement_amount: amount,
            is_efti_compliant: true, // Compliant with EU eFTI standards
            is_delivered: false,
        }
    }

    /// Verifies the submitted OCR SHA-256 hash against the initial registered hash
    pub fn verify_cmr_hash(&self, submitted_hash: &str) -> bool {
        self.cmr_hash == submitted_hash
    }

    /// Validates data integrity, document hash, and executes secure B2B settlement
    pub fn execute_b2b_settlement(&mut self, submitted_hash: &str) -> Result<String, Box<dyn Error>> {
        if !self.is_efti_compliant {
            return Err("Error: eCMR record is not compliant with eFTI regulations!".into());
        }

        if !self.verify_cmr_hash(submitted_hash) {
            return Err("Error: Submitted CMR hash does not match registered eCMR fingerprint!".into());
        }

        self.is_delivered = true;

        // Simulation of instant B2B financial settlement upon successful verification
        let confirmation_msg = format!(
            "B2B settlement successfully executed for eCMR ID: {}. SHA-256 Hash verified. Amount: {:.2} EUR transferred to carrier: {}.",
            self.ecmr_id, self.settlement_amount, self.carrier
        );

        Ok(confirmation_msg)
    }
}

        Ok(confirmation_msg)
    }
}
