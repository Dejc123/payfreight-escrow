/**
 * Project Name: PayFreight Escrow Protocol
 * Description: Universal backend server for secure logistics escrow settlements, 
 * mapping generic external transaction IDs and participant identifiers to a decentralized or secure ledger.
 * 
 * Copyright (c) 2026 PayFreight. All rights reserved.
 * Licensed under the MIT License.
 */

const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');

const app = express();
app.use(cors());
app.use(bodyParser.json());

// In-memory transaction storage (replace with a database or smart contract in production)
const escrowTransactions = [];

// Endpoint to confirm and initialize escrow via the protocol widget
app.post('/api/payfreight/confirm', (req, res) => {
    const { externalId, participantId } = req.body;

    if (!externalId || !participantId) {
        return res.status(400).json({ success: false, message: 'Missing required protocol parameters (externalId or participantId).' });
    }

    const transaction = {
        id: 'PF-' + Date.now(),
        externalId,    // Generic ID representing the order, load, or contract from any external system
        participantId, // ID of the entity/user confirming the escrow process
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
