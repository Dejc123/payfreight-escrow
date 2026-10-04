/*
 * ============================================================================
 * PROJECT: PayFreight Protocol
 * MODULE: Wallet Management & Keypair Utility
 * 
 * COPYRIGHT NOTICE:
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * 
 * PROPRIETARY & CONFIDENTIAL:
 * This source code, its concepts, and architecture are strictly proprietary 
 * and confidential. Unauthorized copying, distribution, or modification of this 
 * file, via any medium, is strictly prohibited.
 * ============================================================================
 * 
 * ABOUT THIS MODULE:
 * This utility module is designed for the PayFreight B2B Logistics Settlement 
 * Protocol built on the Solana blockchain. It provides standardized cryptographic 
 * key management (Keypair loading and generation) required by backend services 
 * and automated test suites.
 * 
 * TARGET AUDIENCE & USERS:
 * Designed specifically for logistics carriers, freight shippers, and platform 
 * administrators participating in trustless, instantaneous on-chain escrow 
 * settlements and token-backed fee discount structures.
 * ============================================================================
*/

import { Keypair } from "@solana/web3.js";
import * as fs from "fs";

/**
 * Helper to load a Solana keypair from a local file path (e.g. CLI configuration)
 */
export function loadWalletFromFile(filePath: string): Keypair {
  try {
    const fileData = fs.readFileSync(filePath, { encoding: "utf-8" });
    const secretKey = Uint8Array.from(JSON.parse(fileData));
    return Keypair.fromSecretKey(secretKey);
  } catch (error) {
    throw new Error(`Failed to load wallet keypair from ${filePath}: ${error}`);
  }
}

/**
 * Generates a temporary fresh keypair for testing purposes or ephemeral signing
 */
export function generateTestWallet(): Keypair {
  return Keypair.generate();
}
