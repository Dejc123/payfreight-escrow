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
                const trimmed = data.trim();
                // Preverimo, če strežnik vrača napako ali besedilo (npr. začetek s črko 'A' ali HTML oznako)
                if (trimmed.startsWith('<') || trimmed.startsWith('A') || trimmed.startsWith('Error')) {
                    return reject(new Error('CargoTrans strežnik vrača napako/HTML namesto JSON-a: ' + trimmed.substring(0, 150)));
                }
                try {
                    const parsed = JSON.parse(data);
                    if (Array.isArray(parsed)) {
                        resolve(parsed);
                    } else {
                        reject(new Error('Prejeti podatki niso v obliki seznama (array).'));
                    }
                } catch (e) {
                    reject(new Error('Napaka pri parsanju JSON-a. Prejeto besedilo se začne z: ' + trimmed.substring(0, 50)));
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
                        weight: offer.weight && offer.weight.trim() !== '' ? parseFloat(offer.weight) : null,
                        length: offer.dimensions && offer.dimensions.trim() !== '' ? parseFloat(offer.dimensions) : null,
                        loading_date: offer.loading_date || null
                    }
                ], { onConflict: ['cargotrans_id'] });
        }

        return res.status(200).json({ success: true, count: offers.length, message: 'Sinhronizacija v Supabase je uspešno zaključena!' });
    } catch (error) {
        return res.status(500).json({ success: false, error: error.message });
    }
};
