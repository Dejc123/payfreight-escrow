import { Resend } from 'resend';

const resend = new Resend(process.env.RESEND_API_KEY);

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ message: 'Method Not Allowed' });
  }

  const { Podjetje, Email, UporabniskoIme, Geslo } = req.body;

  // Generiranje enoličnega verifikacijskega žetona
  const verificationToken = crypto.randomUUID();
  const verificationLink = `https://www.payfreight.io/cargotrans/welcome.html?token=${verificationToken}&UporabniskoIme=${encodeURIComponent(UporabniskoIme)}&Podjetje=${encodeURIComponent(Podjetje)}`;

  try {
    const data = await resend.emails.send({
      from: 'CargoNet Verification <noreply@payfreight.io>',
      to: [Email],
      subject: 'Welcome to CargoNet - Confirm Your Account',
      html: `
        <div style="font-family: sans-serif; background: #0b0f19; color: #ffffff; padding: 30px; border-radius: 12px;">
          <h2 style="color: #22d3ee;">Welcome aboard, ${Podjetje}!</h2>
          <p style="color: #94a3b8;">Your registration on the CargoNet Freight Exchange has been successfully initialized.</p>
          
          <div style="background: #0f172a; border: 1px solid #1e293b; padding: 15px; border-radius: 8px; margin: 20px 0;">
            <p style="margin: 5px 0;"><strong>Username:</strong> ${UporabniskoIme}</p>
            <p style="margin: 5px 0;"><strong>Password:</strong> ${Geslo}</p>
          </div>

          <a href="${verificationLink}" style="display: inline-block; background: #2563eb; color: #ffffff; text-decoration: none; padding: 12px 24px; border-radius: 8px; font-weight: bold;">
            Confirm Account & Activate Access
          </a>
        </div>
      `,
    });

    return res.status(200).json({ success: true, data });
  } catch (error) {
    console.error('Resend Error:', error);
    return res.status(500).json({ success: false, error: error.message });
  }
}
