const https = require('https');
const { createClient } = require('@supabase/supabase-js');

// 1. STIKALO ZA IZKLOP/VKLOP (Nastavite na false, če želite ustaviti uvoz)
const SYNC_ENABLED = true; 

// Nastavitve za Supabase (tukaj vstavite svoje podatke iz Supabase nastavitvah)
const supabaseUrl = process.env.SUPABASE_URL || 'VAS_SUPABASE_URL';
const supabaseKey = process.env.SUPABASE_ANON_KEY || 'VAS_SUPABASE_ANON_KEY';
const supabase = createClient(supabaseUrl, supabaseKey);

// Funkcija, ki povleče podatke iz starčka (ASP)
function fetchCargoOffers(callback) {
    const url = 'https://www.cargotrans.net/export_offers.asp';
    https.get(url, (res) => {
        let data = '';
        res.on('data', (chunk) => { data += chunk; });
        res.on('end', () => {
            try {
                const offers = JSON.parse(data);
                callback(null, offers);
            } catch (e) {
                callback(e, null);
            }
        });
    }).on('error', (err) => {
        callback(err, null);
    });
}

// Glavna funkcija za uvoz v Supabase
async function runSync() {
    if (!SYNC_ENABLED) {
        console.log("Sinhronizacija tovora je trenutno izklopljena (stikalo SYNC_ENABLED je false).");
        return;
    }

    console.log("Začenjam uvoz ponudb iz CargoTrans ASP...");
    
    fetchCargoOffers(async (err, offers) => {
        if (err) {
            console.error("Napaka pri branju s stare strani:", err);
            return;
        }

        for (let offer of offers) {
            // Zapišemo v Supabase tabelo enter_cargo
            const { error } = await supabase
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

            if (error) {
                console.error(`Napaka pri zapisu za tovor ID ${offer.tovor_id}:`, error.message);
            }
        }
        console.log("Uvoz ponudb je uspešno zaključen!");
    });
}

module.exports = { runSync };
