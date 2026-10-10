/**
 * ============================================================================
 * PAYFREIGHT PROTOCOL – ESCROW MAIL API ENDPOINT (VERCEL SERVERLESS)
 * ============================================================================
 * Copyright (c) 2026 PayFreight Protocol. All rights reserved.
 * Proprietary and Confidential. Unauthorized copying, distribution, or use 
 * of this file via any medium is strictly prohibited.
 * ============================================================================
 * Description:
 * Vercel serverless function endpoint responsible for triggering carrier fee 
 * requests via the PayFreightMailer engine. Accepts POST requests containing 
 * order details, secures transactions, and routes replies appropriately.
 * ============================================================================
 */

const PayFreightMailer = require('../mailer');
const mailer = new PayFreightMailer(process.env.RESEND_API_KEY);

export default async function handler(req, res) {
    if (req.method !== 'POST') {
        return res.status(405).json({ error: 'Method not allowed' });
    }

    try {
        const { carrierEmail, orderId, loadId, amount, secureToken, replyTo } = req.body;

        // Call the mailer method with support for custom reply-to routing
        const response = await mailer.sendCarrierFeeRequest(
            carrierEmail, 
            orderId, 
            loadId, 
            amount, 
            secureToken,
            replyTo || 'support@payfreight.io'
        );

        return res.status(200).json({ success: true, data: response });
    } catch (error) {
        console.error('API Error:', error);
        return res.status(500).json({ error: error.message });
    }
}
