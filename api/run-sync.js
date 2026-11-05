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
                // Če se začne z HTML oznako ali napako, takoj vrnemo jasen error
                if (data.trim().startsWith('<') || data.trim().startsWith('A')) {
                    return reject(new Error('CargoTrans vrača HTML/Napako namesto JSON-a: ' + data.substring(0, 150)));
                }
                try {
                    const parsed = JSON.parse(data);
                    resolve(parsed);
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
                        cargotrans_id: String(offer.tovor_id),
                        origin_city: offer.kraj_odhod || null,
                        dest_city: offer.kraj_prihod || null,
                        weight: offer.weight ? parseFloat(offer.weight) : null,
                        length: offer.dimensions ? parseFloat(offer.dimensions) : null
                    }
                ], { onConflict: ['cargotrans_id'] });
        }

        return res.status(200).json({ success: true, count: offers.length, message: 'Sinhronizacija v Supabase je uspešno zaključena!' });
    } catch (error) {
        return res.status(500).json({ success: false, error: error.message });
    }
};
    } catch (error) {
        return res.status(500).json({ success: false, error: error.message });
    }
};
