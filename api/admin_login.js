export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { username, password } = req.body;

    // Trdo določeni tajni podatki na strežniški strani (nevidni za javnost)
    const SECRET_ADMIN_USER = process.env.ADMIN_USER || "DejanLuzar";
    const SECRET_ADMIN_PASS = process.env.ADMIN_PASS || "Dluzar12345!";

    if (username === SECRET_ADMIN_USER && password === SECRET_ADMIN_PASS) {
      // Uspešna prijava – vrnemo varen žeton seje
      return res.status(200).json({
        success: true,
        token: "SUPER_ADMIN_GRANTED_PAYFREIGHT_2026"
      });
    } else {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
  } catch (error) {
    return res.status(500).json({ error: 'Server authentication error' });
  }
}
