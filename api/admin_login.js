import speakeasy from 'speakeasy';

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { username, password, token2FA } = req.body;

    // 1. Nastavitve podatkov na strežniku
    const SECRET_ADMIN_USER = process.env.ADMIN_USER || "DejanLuzar";
    const SECRET_ADMIN_PASS = process.env.ADMIN_PASS || "Dluzar12345!";
    
    // Tvoj tajni 2FA ključ (Base32 format) - Vnesi svoj poljubni tajni niz ali ga generiraj
    const SECRET_2FA_BASE32 = process.env.ADMIN_2FA_SECRET || "KVKFKRCPNZQUYMLXOVYDSQKJKZDTSRLD";

    // 2. Preverjanje uporabniškega imena in gesla
    if (username !== SECRET_ADMIN_USER || password !== SECRET_ADMIN_PASS) {
      return res.status(401).json({ error: 'Neveljavno uporabniško ime ali geslo.' });
    }

    // 3. Preverjanje 6-mestne Google Authenticator kode
    if (!token2FA) {
      return res.status(400).json({ error: 'Vnesti morate 2FA kodo iz Google Authenticatorja.' });
    }

    const verified = speakeasy.totp.verify({
      secret: SECRET_2FA_BASE32,
      encoding: 'base32',
      token: token2FA,
      window: 1 // Dovoljuje odstopanje ±30 sekund zaradi časovnih razlik
    });

    if (!verified) {
      return res.status(401).json({ error: 'Neveljavna ali potekla 2FA koda!' });
    }

    // Uspešna autentikacija z 2FA
    return res.status(200).json({
      success: true,
      token: "SUPER_ADMIN_2FA_VERIFIED_PAYFREIGHT_2026"
    });

  } catch (error) {
    return res.status(500).json({ error: 'Strežniška napaka pri preverjanju 2FA.' });
  }
}
