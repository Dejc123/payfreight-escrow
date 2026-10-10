// api/send-escrow-mail.js
const PayFreightMailer = require('../mailer'); // Prilagodi pot do tvojega mailerja
const mailer = new PayFreightMailer(process.env.RESEND_API_KEY);

export default async function handler(req, res) {
    if (req.method !== 'POST') {
        return res.status(405).json({ error: 'Method not allowed' });
    }

    try {
        const { carrierEmail, orderId, loadId, amount, secureToken } = req.body;

        // Klic metode iz tvojega mailerja
        const response = await mailer.sendCarrierFeeRequest(
            carrierEmail, 
            orderId, 
            loadId, 
            amount, 
            secureToken
        );

        return res.status(200).json({ success: true, data: response });
    } catch (error) {
        console.error('API Error:', error);
        return res.status(500).json({ error: error.message });
    }
}
