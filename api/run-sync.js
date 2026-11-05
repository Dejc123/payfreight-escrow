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
                        cargotrans_id: String(offer.tovor_id),
                        origin_city: offer.kraj_odhod || 'Neznano',
                        dest_city: offer.kraj_prihod || 'Neznano',
                        weight: offer.weight ? parseFloat(offer.weight) : null,
                        length: offer.dimensions ? parseFloat(offer.dimensions) : null,
                        loading_date: offer.loading_date || null,
                        transport_mode: 'road', // Nastavljeno na privzeti cestni prevoz, da dashboard ne filtrira praznih vrstic
                        origin_country: 'SI',   // Privzeta oznaka, da filtri za državo ne izločijo vnosa
                        dest_country: 'EU',     // Privzeta oznaka
                        company_name: offer.company_name || 'CargoTrans Partner',
                        contact_name: offer.contact_name || '',
                        contact_phone: offer.contact_phone || ''
                    }
                ], { onConflict: ['cargotrans_id'] });
        }

        return res.status(200).json({ success: true, count: offers.length, message: 'Sinhronizacija uspešno zaključena in prilagojena za prikaz!' });
    } catch (error) {
        return res.status(500).json({ success: false, error: error.message });
    }
};
