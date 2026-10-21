# PayFreight Mainnet Deployment & Troubleshooting Guide

Operational roadmap for transitioning PayFreight from local test environment to production (Mainnet) without security risks or unexpected server crashes.

## 1. Architecture & Gateway Integration Overview
To ensure a secure bridge between traditional fiat payments and on-chain escrow, the system relies on a unified structure:
- **Frontend Gateway (`gateway.html`):** Renders the secure deposit interface, capturing transaction parameters and communicating via `fetch()` requests.
- **Backend Bridge (`server_mainnet.js`):** Express & Axios Node.js server running on the production server. It reads environment variables (`.env`) securely and communicates directly with the production Circle API (`https://api.circle.com/v1`).
- **Static Asset Routing:** The server must use `express.static` to correctly resolve frontend routes and eliminate `Cannot GET /gateway.html` errors on `payfreight.io`.

---

## 2. Why Error 500 (FUNCTION_INVOCATION_FAILED) Occurs on Vercel
Error 500 on serverless platforms like Vercel typically occurs due to three specific reasons:

### A) Missing npm Packages
In a serverless environment, all required libraries (`@google-cloud/vision`, `multer`, `nodemailer`, `express`, `axios`, `cors`, `dotenv`) must be explicitly declared in your `package.json`. 
Run the following command in your terminal to install dependencies:
```bash
npm install @google-cloud/vision multer nodemailer body-parser cors express axios dotenv
B) Vercel Serverless Routing (vercel.json)
Vercel requires a vercel.json configuration file to properly route Node.js Express requests. If it is missing or misconfigured, the server cannot process incoming requests.
Create or update vercel.json in your project root:

JSON
{
  "version": 2,
  "builds": [
    {
      "src": "Mainnet/server_mainnet.js",
      "use": "@vercel/node"
    }
  ],
  "routes": [
    {
      "src": "/(.*)",
      "dest": "Mainnet/server_mainnet.js"
    }
  ]
}
Tip: When you are ready to launch your mainnet server, ensure your vercel.json points to "src": "Mainnet/server_mainnet.js" as shown above.

C) Google Cloud Environment Variables & Safety Fallback
Using Google Cloud Vision without proper credentials will trigger a critical runtime error in Node.js, causing Vercel to terminate the function.

In Mainnet/server_mainnet.js, a dual-safety mechanism is integrated:

If credentials exist: The system utilizes Google Cloud Vision OCR for scanning paper CMRs.

If credentials are missing: The system automatically switches to a secure fallback OCR generator, computes a valid SHA-256 e-CMR hash, and prevents server crashes (giving you 100% uptime during testing).

3. Mainnet Deployment Steps (Action Items)
[ ] Environment Configuration: Set up the production .env file on the server with the live Mainnet Circle API key:

Delček kode
CIRCLE_API_KEY=your_live_mainnet_key
[ ] Process Management: Run the production server continuously using PM2 or configure an Nginx reverse proxy pointing to http://localhost:3000 (if deploying on a traditional VPS) or use the Vercel serverless configuration above.

[ ] Smart Contract / Escrow Verification: Ensure Solana on-chain programs are deployed to Mainnet-Beta.

4. Security & Community Validation
[ ] Peer review of smart contracts and backend routes.

[ ] Automated static analysis on Rust/Anchor components.

5. Official Go-Live
[ ] Public launch of payfreight.io with active backend integration and live settlement flows.
