const https = require('https');
const { createClient } = require('@supabase/supabase-js');

const supabaseUrl = process.env.SUPABASE_URL;
const supabaseKey = process.env.SUPABASE_ANON_KEY;
const supabase = createClient(supabaseUrl, supabaseKey);

function fetchCargoOffers() {
    return new Promise((resolve, reject) => {
        // Tukaj je sedaj dodan pravilen varnostni ključ, ki ga zahteva CargoTrans
        const url = 'https://www.cargotrans.net/export_offers.asp?key=cargotrans_secure_sync_190877';
        
        https.get(url, (res) => {
            let data = '';
            res.on('data', (chunk) => { data += chunk; });
            res.on('end', () => {
                try {
                    resolve(JSON.parse(data));
                } catch (e) {
                    reject(new Error('Napaka pri parsanju JSON-a (prejeto: ' + data.substring(0, 100) + '...)'));
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
                        origin_country: offer.drzava_odhod_oznaka,
                        origin_city: offer.kraj_odhod,
                        dest_country: offer.drzava_prihod_oznaka,
                        dest_city: offer.kraj_prihod,
                        weight: offer.teza,
                        dimensions: offer.dimenzije,
                        vehicle_type: offer.nadgradnja
                    }
                ], { onConflict: ['cargotrans_id'] });
        }

        return res.status(200).json({ success: true, count: offers.length, message: 'Sinhronizacija v Supabase je uspešno zaključena!' });
    } catch (error) {
        return res.status(500).json({ success: false, error: error.message });
    }
};
