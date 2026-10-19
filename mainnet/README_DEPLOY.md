# PayFreight Mainnet Deployment & Troubleshooting Guide

## Why Error 500 (FUNCTION_INVOCATION_FAILED) Occurs on Vercel
Error 500 on serverless platforms like Vercel typically occurs due to three specific reasons:

### A) Missing npm Packages
In a serverless environment, all required libraries (`@google-cloud/vision`, `multer`, `nodemailer`) must be explicitly declared in your `package.json`. 
Run the following command in your terminal to install dependencies:
```bash
npm install @google-cloud/vision multer nodemailer body-parser cors express

### B) Vercel Serverless Routing (`vercel.json`)
Vercel requires a vercel.json configuration file to properly route Node.js Express requests. If it is missing or misconfigured, the server cannot process incoming requests.
Create or update vercel.json in your project root:

```json
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

----------------------------------------------------------------------------------------------------------------------------------

In Mainnet/server_mainnet.js, a dual-safety mechanism is integrated:

If credentials exist: The system utilizes Google Cloud Vision OCR for scanning paper CMRs.

If credentials are missing: The system automatically switches to a secure fallback OCR generator, computes a valid SHA-256 e-CMR hash, and prevents server crashes (giving you 100% uptime during testing).
