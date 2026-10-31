import { createClient } from '@supabase/supabase-js';

const SUPABASE_URL = process.env.SUPABASE_URL || 'https://wqjqjfxyucufuyioxphm.supabase.co';
const SUPABASE_KEY = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY;
const supabase = createClient(SUPABASE_URL, SUPABASE_KEY);

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const { cargoTransId, firstName, lastName, email, role, access, username, password, mobile, office } = req.body;

    if (!cargoTransId || !username || !password || !email) {
      return res.status(400).json({ error: 'CargoTransID, Email, Username and Password are required.' });
    }

    // Vnos člana ekipe v Supabase bazo 'team_members' za CargoNet
    const { data, error } = await supabase
      .from('team_members')
      .insert([
        {
          cargo_trans_id: cargoTransId,
          first_name: firstName,
          last_name: lastName,
          email: email,
          role: role,
          access_level: access,
          username: username,
          password: password,
          mobile: mobile,
          office_ext: office,
          platform: 'cargonet',
          created_at: new Date().toISOString()
        }
      ]);

    if (error) {
      console.error("Supabase insert error (CargoNet):", error);
      return res.status(500).json({ error: error.message });
    }

    return res.status(200).json({
      success: true,
      message: 'CargoNet team member account created successfully.',
      member: { username, email, cargoTransId }
    });

  } catch (error) {
    console.error("Server error:", error);
    return res.status(500).json({ error: error.message });
  }
}
