/**
 * Project Name: PayFreight Escrow Protocol
 * Description: Universal backend server for secure logistics escrow settlements.
 * 
 * Copyright (c) 2026 Payfreight / Vse pravice pridržane.
 */

const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');
const path = require('path');

const app = express();
app.use(cors());
app.use(bodyParser.json());

// Serve static frontend files from the 'public' folder
app.use(express.static(path.join(__dirname, 'public')));

const escrowTransactions = [];

app.post('/api/payfreight/confirm', (req, res) => {
    const { externalId, participantId } = req.body;

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
