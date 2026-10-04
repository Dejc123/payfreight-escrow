// =========================================================================
// MODULE: PayFreight eCMR Compliance & eIDAS Digital Signature
// LICENSE: Copyright (c) 2026 PayFreight. All rights reserved.
// AUTHOR: PayFreight Core Development Team
//
// DESCRIPTION:
// This module provides advanced electronic signature capabilities compliant
// with EU eIDAS regulations for digital consignment notes (eCMR). It hashes
// the standardized freight data using SHA-256 and signs the resulting hash
// using Ed25519 keypairs, creating an immutable cryptographic proof before
// triggering Solana smart contract payouts.
// =========================================================================

use ed25519_dalek::{SigningKey, Signer, Signature};
use sha2::{Sha256, Digest};

/// Takes raw eCMR document data and cryptographically signs it using a private key.
/// This ensures that the data cannot be altered after signing (EU Compliance).
pub fn process_ecmr_signature(ecmr_data: &[u8], private_key: &[u8; 32]) -> Signature {
    // 1. Create a unique cryptographic hash of the eCMR data
    let mut hasher = Sha256::new();
    hasher.update(ecmr_data);
    let ecmr_hash = hasher.finalize();

    // 2. Create the signing key from the raw bytes
    let signing_key = SigningKey::from_bytes(private_key);
    
    // 3. Sign the hash (Advanced Electronic Signature)
    signing_key.sign(&ecmr_hash)
}
