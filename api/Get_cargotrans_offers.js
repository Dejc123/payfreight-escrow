
const { createClient } = require('@supabase/supabase-js');

module.exports = async (req, res) => {
    if (req.method !== 'GET') {
        return res.status(405).json({
            success: false,
            error: 'Method not allowed'
        });
    }

    try {
        const supabaseUrl = process.env.SUPABASE_URL;
        const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY;

        if (!supabaseUrl || !supabaseKey) {
            throw new Error('Manjka Supabase konfiguracija.');
        }

        const supabase = createClient(supabaseUrl, supabaseKey);

        const { data, error } = await supabase
            .from('cargotrans_offers')
            .select('*')
            .order('loading_date', { ascending: true });

        if (error) {
            throw new Error(error.message);
        }

        return res.status(200).json({
            success: true,
            count: data.length,
            offers: data
        });

    } catch (error) {
        console.error('CargoTrans offers read error:', error);

        return res.status(500).json({
            success: false,
            error: error.message
        });
    }
};

