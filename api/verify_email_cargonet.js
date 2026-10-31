import { Resend } from 'resend';

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { token } = req.body;

  if (!token) {
    return res.status(400).json({ error: 'Missing verification token' });
  }

  // Tukaj v bazi/seji poiščeš uporabnika s tem token-om in nastaviš isVerified = true

  return res.status(200).json({ success: true, message: 'Email verified successfully' });
}
