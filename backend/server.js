require('dotenv').config();
const express = require('express');
const axios = require('axios');
const cors = require('cors');
const path = require('path');

const app = express();
app.use(express.json());
app.use(cors());

// Serve static frontend files (HTML, CSS) from the 'public' folder
app.use(express.static(path.join(__dirname, '../public')));

// Verify if the API key is available in the environment variables
const CIRCLE_API_KEY = process.env.CIRCLE_API_KEY;
const CIRCLE_BASE_URL = 'https://api-sandbox.circle.com/v1'; // For Testnet (Sandbox)

if (!CIRCLE_API_KEY) {
    console.error("Error: Missing CIRCLE_API_KEY in the .env file!");
    process.exit(1);
}

// Endpoint called by your frontend when an order is submitted
app.post('/api/create-payment', async (req, res) => {
    try {
        const { amount, currency, carrier, reference } = req.body;

        // Call the official Circle API to create a bank deposit instruction
        const response = await axios.post(
            `${CIRCLE_BASE_URL}/businessAccount/deposits/bank/instructions`,
            {
                idempotencyKey: require('crypto').randomUUID(),
                amount: {
                    amount: amount.toString(),
                    currency: currency || 'EUR'
                },
                // Additional tracking metadata for freight and carrier
                metadata: {
                    carrier: carrier,
                    freightReference: reference
                }
            },
            {
                headers: {
                    'Authorization': `Bearer ${CIRCLE_API_KEY}`,
                    'Content-Type': 'application/json'
                }
            }
        );

        res.status(200).json({
            success: true,
            data: response.data
        });

    } catch (error) {
        console.error("Error communicating with Circle API:", error.response?.data || error.message);
        res.status(500).json({
            success: false,
            error: error.response?.data || 'Server error'
        });
    }
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
    console.log(`PayFreight backend is running on http://localhost:${PORT}`);
});
