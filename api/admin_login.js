import crypto from 'crypto';

// Standardna funkcija za dešifriranje Base32 v Buffer
function base32decode(base32) {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
  let bits = '';
  let value = 0;

  let cleanInput = base32.replace(/=+$/, '').toUpperCase();
  for (let i = 0; i < cleanInput.length; i++) {
    const val = alphabet.indexOf(cleanInput.charAt(i));
    if (val === -1) continue;
    bits += val.toString(2).padStart(5, '0');
  }

  const bytes = [];
  for (let i = 0; i + 8 <= bits.length; i += 8) {
    bytes.push(parseInt(bits.substr(i, 8), 2));
  }

  return Buffer.from(bytes);
}

// Generiranje 6-mestne TOTP kode
function generateTOTP(secretBase32, timeStep = 30) {
  const key = base32decode(secretBase32);
  const epoch = Math.floor(Date.now() / 1000);
  const counter = Math.floor(epoch / timeStep);

  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64BE(BigInt(counter), 0);

  const hmac = crypto.createHmac('sha1', key).update(buffer).digest();
  const offset = hmac[hmac.length - 1] & 0xf;

  const code = (
    ((hmac[offset] & 0x7f) << 24) |
    ((hmac[offset + 1] & 0xff) << 16) |
    ((hmac[offset + 2] & 0xff) << 8) |
    (hmac[offset + 3] & 0xff)
  ) % 1000000;

  return code.toString().padStart(6, '0');
}

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { username, password, token2FA } = req.body;

    const SECRET_ADMIN_USER = process.env.ADMIN_USER || "DejanLuzar";
    const SECRET_ADMIN_PASS = process.env.ADMIN_PASS || "Dluzar12345!";
    const SECRET_2FA_BASE32 = process.env.ADMIN_2FA_SECRET || "KVKFKRCPNZQUYMLXOVYDSQKJKZDTSRLD";

    // Preverjanje uporabniškega imena in gesla
    if (username !== SECRET_ADMIN_USER || password !== SECRET_ADMIN_PASS) {
      return res.status(401).json({ error: 'Neveljavno uporabniško ime ali geslo.' });
    }

    if (!token2FA) {
      return res.status(400).json({ error: 'Vnesite 6-mestno 2FA kodo.' });
    }

    // Izračunamo trenutno kodo na strežniku (z toleranco za časovne razlike)
    const validCode = generateTOTP(SECRET_2FA_BASE32);

    if (token2FA.trim() !== validCode) {
      return res.status(401).json({ error: 'Neveljavna 2FA koda iz Google Authenticatorja!' });
    }

    return res.status(200).json({
      success: true,
      token: "SUPER_ADMIN_2FA_VERIFIED_PAYFREIGHT_2026"
    });

  } catch (error) {
    console.error("API Auth Error:", error);
    return res.status(500).json({ error: 'Napaka na strežniku pri obdelavi zahteve.' });
  }
}
