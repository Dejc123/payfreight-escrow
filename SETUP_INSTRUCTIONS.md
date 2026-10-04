cat << 'EOF' > SETUP_INSTRUCTIONS.md
<!--
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Author: PayFreight Protocol Team
 * This document is proprietary and confidential.
-->

# 🚀 PayFreight Protocol – Complete Setup, Instructions & Wallet Helper

This file contains everything in one place: complete deployment instructions from Step 1 to Step 7, and the required wallet utility code.

---

## PART 1: Instructions (Step 1 to Step 7)

### Step 1: Repository Cloning & Dependencies
Open your terminal, clone the repository, navigate into the project directory, and install all required Node.js dependencies:
```bash
git clone [https://github.com/Dejc123/payfreight-escrow.git](https://github.com/Dejc123/payfreight-escrow.git)
cd payfreight-escrow
npm install
----------------------------------------------
Step 2: Solana Localnet Configuration
Configure the Solana CLI to point to your local development cluster:

Bash
solana config set --url localhost
Step 3: Smart Contract Build
Build the smart contract using the Anchor framework to compile the source code into bytecode:

Bash
anchor build
Step 4: Local Validator & Testing
In a separate terminal window, start the local blockchain validator:

Bash
solana-test-validator
In your original terminal window, run the test suite to verify the escrow logic and dynamic fee tiers:

Bash
anchor test
Step 5: Backend & Web2.5 Environment Setup
Create a file named .env in the root directory of the project and add your configuration variables:

Delček kode
SOLANA_RPC_URL=[https://api.devnet.solana.com](https://api.devnet.solana.com)
PRIVY_APP_ID=your_privy_app_id
PRIVY_SECRET=your_privy_secret
DISCOUNT_TOKEN_MINT=PayfreightUtilityTokenMintAddress11111111111
Step 6: Application Startup
Once everything is configured, start the backend server to handle freight settlements and e-PoD verifications:

Bash
node server.js
Step 7: Final Verification & Deployment
Verify that all services are running correctly and that the local or devnet cluster is responding to transaction requests before presenting or deploying the repository.
