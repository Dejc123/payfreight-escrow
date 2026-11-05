const https = require('https');
const { createClient } = require('@supabase/supabase-js');

const supabaseUrl = process.env.SUPABASE_URL;
const supabaseKey = process.env.SUPABASE_ANON_KEY;
const supabase = createClient(supabaseUrl, supabaseKey);

function fetchCargoOffers() {
    return new Promise((resolve, reject) => {
        const url = 'https://www.cargotrans.net/export_offers.asp?key=cargotrans_secure_sync_190877';
        
        https.get(url, (res) => {
            let data = '';
            res.on('data', (chunk) => { data += chunk; });
            res.on('end', () => {
                try {
                    const parsed = JSON.parse(data);
                    if (Array.isArray(parsed)) {
                        resolve(parsed);
                    } else if (parsed && parsed.error) {
                        reject(new Error('CargoTrans napaka: ' + parsed.error));
                    } else {
                        reject(new Error('Neveljaven format podatkov (ni seznam).'));
                    }
                } catch (e) {
                    reject(new Error('Napaka pri parsanju JSON-a.'));
                }
            });
        }).on('error', (err) => {
            reject(err);
        });
    });
}

module.exports = async (req, res) => {
    try {
        const offers = await fetchCargoOffers();

        for (let offer of offers) {
            await supabase
                .from('enter_cargo')
                .upsert([
                    { 
                        cargotrans_id: offer.tovor_id,
                        origin_city: offer.kraj_odhod,
                        dest_city: offer.kraj_prihod,
                        weight: offer.weight ? parseFloat(offer.weight) : null,
                        dimensions: offer.dimensions,
                        loading_date: offer.loading_date || null,
                        company_name: offer.company_name,
                        contact_name: offer.contact_name,
                        contact_phone: offer.contact_phone
                    }
                ], { onConflict: ['cargotrans_id'] });
        }

        return res.status(200).json({ success: true, count: offers.length, message: 'Sinhronizacija in prenos podatkov o ponudniku sta uspela!' });
    } catch (error) {
        return res.status(500).json({ success: false, error: error.message });
    }
};
