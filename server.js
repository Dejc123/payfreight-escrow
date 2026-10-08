/**
 * Project Name: PayFreight Escrow Protocol
 * Description: Universal backend server for secure logistics escrow settlements.
 * 
 * Copyright (c) 2026 Payfreight / All rights reserved.
 */

const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');
const path = require('path');
const nodemailer = require('nodemailer');

const app = express();
app.use(cors());
app.use(bodyParser.json());

// Serve static frontend files from the 'public' folder
app.use(express.static(path.join(__dirname, 'public')));

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
