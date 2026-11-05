const express = require('express');
const cors = require('cors');
const { sendEmail } = require('./mailer');
const { fetchCargoOffers } = require('./fetch_offers');
const { runSync } = require('./sync');

const app = express();
const PORT = process.env.PORT || 3000;

// Server settings
app.use(cors());
app.use(express.json());

// Test route to check if server is running
app.get('/', (req, res) => {
  res.json({ status: 'CargoTrans Backend is active and running!' });
});

// Endpoint to send rating invitation emails
app.post('/api/send-rating-invite', async (req, res) => {
  const { carrierEmail, dealId, carrierName } = req.body;

  if (!carrierEmail || !dealId) {
    return res.status(400).json({ success: false, error: 'Missing required fields (carrierEmail or dealId).' });
  }

  const ratingLink = `https://cargotrans.io/rate_deal.html?deal=${dealId}&target=carrier`;
  
  const htmlBody = `
    <div style="font-family: sans-serif; padding: 20px; background: #0f172a; color: #fff; border-radius: 10px;">
      <h2>Hello, ${carrierName || 'Partner'}</h2>
      <p>Your transport deal (ID: ${dealId}) has been successfully completed.</p>
      <p>Please take a moment to rate your collaboration:</p>
      <a href="${ratingLink}" style="display: inline-block; padding: 12px 24px; background: #7c3aed; color: #fff; text-decoration: none; border-radius: 8px; font-weight: bold; margin-top: 10px;">Rate Deal Now</a>
    </div>
  `;

  const sent = await sendEmail(carrierEmail, `Rate completed transport deal #${dealId}`, htmlBody);

  if (sent) {
    res.json({ success: true, message: 'Rating invitation email successfully sent.' });
  } else {
    res.status(500).json({ success: false, error: 'Error sending email.' });
  }
});

// Endpoint za prikaz tovornih ponudb iz CargoTrans-a
app.get('/api/cargotrans-offers', (req, res) => {
    fetchCargoOffers((err, offers) => {
        if (err) {
            return res.status(500).json({ success: false, error: 'Failed to fetch offers from CargoTrans ASP server.' });
        }
        res.json({ success: true, count: offers.length, data: offers });
    });
});

// Endpoint za ročni zagon sinhronizacije s starim ASP sistemom v Supabase
app.get('/api/run-sync', async (req, res) => {
    try {
        await runSync();
        res.json({ success: true, message: 'Sinhronizacija tovora v Supabase je bila uspešno sprožena!' });
    } catch (error) {
        res.status(500).json({ success: false, error: error.message });
    }
});

// Start server (Mora biti vedno na koncu!)
app.listen(PORT, () => {
  console.log(`CargoTrans backend server is running on port: ${PORT}`);
});
