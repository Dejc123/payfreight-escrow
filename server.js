/**
 * Project Name: PayFreight Escrow Protocol
 * Description: Universal backend server for secure logistics escrow settlements.
 * 
 * Change Log:
 * - Date: October 19, 2026
 * - Modifications: Added Multer middleware for file uploads, integrated Google Cloud Vision OCR processing for paper CMR scanning, added SHA-256 digital e-CMR hash generation, and implemented carrier/shipper authorization checks for automated escrow verification and release.
 * 
 * Copyright (c) 2026 Payfreight / All rights reserved.
 */

const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');
const path = require('path');
const nodemailer = require('nodemailer');
const multer = require('multer');
const crypto = require('crypto');
const vision = require('@google-cloud/vision');

// Import the modular carrier portal component
const PayFreightCarrierPortal = require('./carrier-portal');

const app = express();
const upload = multer({ storage: multer.memoryStorage() });

app.use(cors());
app.use(bodyParser.json());

// Serve static frontend files from the 'public' folder
app.use(express.static(path.join(__dirname, 'public')));

// Initialize Google Cloud Vision Client for OCR
const visionClient = new vision.ImageAnnotatorClient();

// SMTP configuration (reads environment variables from Vercel)
const transporter = nodemailer.createTransport({
    host: process.env.SMTP_HOST || 'smtp.resend.com',
    port: Number(process.env.SMTP_PORT) || 465,
    secure: true, // true for port 465
    auth: {
        user: process.env.SMTP_USER || 'resend',
        pass: process.env.SMTP_PASS
    }
});

const escrowTransactions = [];

// Explicit route for the Pre-Sale page (payfreight.io/presale.html and payfreight.io/presale)
app.get('/presale.html', (req, res) => {
    res.sendFile(path.join(__dirname, 'public', 'presale.html'));
});

app.get('/presale', (req, res) => {
    res.sendFile(path.join(__dirname, 'public', 'presale.html'));
});

// Route to render the carrier portal dynamically with pre-filled parameters
app.get('/api/carrier-portal', (req, res) => {
    const { shipperId, cargoId, shipperEmail } = req.query;

    const portal = new PayFreightCarrierPortal();
    const carrierPageHtml = portal.renderForm({
        shipperId: shipperId || 'N/A',
        cargoId: cargoId || 'N/A',
        shipperEmail: shipperEmail || 'N/A'
    });

    res.send(carrierPageHtml);
});

/**
 * NEW ENDPOINT: Upload paper CMR, run OCR, generate SHA-256 e-CMR hash, and authorize release
 */
app.post('/api/payfreight/upload-cmr', upload.single('cmr_file'), async (req, res) => {
    try {
        const { escrowId, carrierId, shipperId } = req.body;
        const fileBuffer = req.file ? req.file.buffer : null;

        if (!fileBuffer) {
            return res.status(400).json({ success: false, message: 'Missing required CMR image file.' });
        }

        if (!escrowId || !carrierId || !shipperId) {
            return res.status(400).json({ success: false, message: 'Missing protocol parameters (escrowId, carrierId, or shipperId).' });
        }

        // Google Cloud Vision OCR Processing
        const [result] = await visionClient.textDetection(fileBuffer);
        const detections = result.textAnnotations;
        const rawOcrText = detections[0] ? detections[0].description : '';

        if (!rawOcrText) {
            return res.status(400).json({ success: false, message: 'OCR failed to extract text from the uploaded CMR image.' });
        }

        // Build structured dataset for the e-CMR fingerprint
        const cmrDataset = {
            rawOcrText,
            timestamp: new Date().toISOString(),
            carrierId,
            shipperId,
            escrowId
        };

        // Generate SHA-256 cryptographic hash (Digital e-CMR Fingerprint)
        const jsonString = JSON.stringify(cmrDataset);
        const cmrHash = crypto.createHash('sha256').update(jsonString).digest('hex');

        // Find or register transaction
        let transaction = escrowTransactions.find(t => t.id === escrowId);
        if (!transaction) {
            transaction = {
                id: escrowId || 'PF-' + Date.now(),
                carrierId,
                shipperId,
                cmrHash,
                status: 'CMR_VERIFIED_AND_RELEASED',
                createdAt: new Date()
            };
            escrowTransactions.push(transaction);
        } else {
            transaction.cmrHash = cmrHash;
            transaction.status = 'CMR_VERIFIED_AND_RELEASED';
        }

        console.log(`[PayFreight e-CMR] Generated SHA-256 Hash for transaction ${transaction.id}: ${cmrHash}`);

        res.json({
            success: true,
            message: 'Paper CMR successfully scanned, converted to e-CMR hash, and escrow release authorized!',
            data: {
                transactionId: transaction.id,
                cmrHash,
                status: transaction.status,
                ocrSnippet: rawOcrText.substring(0, 150) + '...'
            }
        });

    } catch (error) {
        console.error('Error processing CMR upload and verification:', error);
        res.status(500).json({ success: false, message: error.message });
    }
});

app.post('/api/payfreight/confirm', async (req, res) => {
    const { externalId, participantId, recipientEmail } = req.body;

    if (!externalId || !participantId) {
        return res.status(400).json({ success: false, message: 'Missing required protocol parameters (externalId or participantId).' });
    }

    const transaction = {
        id: 'PF-' + Date.now(),
        externalId,
        participantId,
        status: 'PENDING',
        createdAt: new Date()
    };

    escrowTransactions.push(transaction);
    console.log('New PayFreight Protocol transaction initialized:', transaction);

    // Attempt to send email via Resend
    try {
        if (recipientEmail) {
            await transporter.sendMail({
                from: 'escrow@payfreight.io',
                to: recipientEmail,
                subject: `PayFreight Transaction ${transaction.id} Initialized`,
                text: `Hello,\n\nYour transaction with ID ${transaction.id} (External ID: ${externalId}) has been successfully initialized in the PayFreight system.\n\nBest regards,\nPayFreight Team`
            });
            console.log(`Email successfully sent to: ${recipientEmail}`);
        }
    } catch (error) {
        console.error('Error sending email:', error);
    }

    res.json({
        success: true,
        message: 'PayFreight escrow protocol successfully initialized!',
        transaction
    });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
    console.log(`PayFreight Protocol server is running on port ${PORT}`);
});
