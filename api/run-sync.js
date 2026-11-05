
const https = require('https');
const { createClient } = require('@supabase/supabase-js');

const supabaseUrl = process.env.SUPABASE_URL;
const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY;

if (!supabaseUrl || !supabaseKey) {
    throw new Error('Manjka SUPABASE_URL ali Supabase ključ.');
}

const supabase = createClient(supabaseUrl, supabaseKey);

const CARGOTRANS_URL =
    'https://www.cargotrans.net/export_offers.asp?key=cargotrans_secure_sync_190877';

function fetchCargoOffers() {
    return new Promise((resolve, reject) => {
        https.get(CARGOTRANS_URL, (res) => {
            let data = '';

            if (res.statusCode !== 200) {
                res.resume();
                return reject(new Error('CargoTrans API HTTP napaka: ' + res.statusCode));
            }

            res.on('data', (chunk) => {
                data += chunk;
            });

            res.on('end', () => {
                try {
                    const parsed = JSON.parse(data);

                    if (!Array.isArray(parsed)) {
                        return reject(new Error('CargoTrans API ni vrnil seznama ponudb.'));
                    }

                    resolve(parsed);
                } catch (error) {
                    reject(new Error(
                        'Napaka pri branju CargoTrans JSON: ' + data.substring(0, 200)
                    ));
                }
            });
        }).on('error', reject);
    });
}

function cleanText(value) {
    if (value === null || value === undefined) {
        return null;
    }

    const cleaned = String(value).trim();

    return cleaned === '' ? null : cleaned;
}

function mapOffer(offer) {
    return {
        tovor_id: cleanText(offer.tovor_id),
        ponudnik_id: cleanText(offer.ponudnik_id),

        podjetje: cleanText(offer.podjetje),
        drzava_ponudnik: cleanText(offer.drzava_ponudnik),

        kraj_odhod: cleanText(offer.kraj_odhod),
        kraj_prihod: cleanText(offer.kraj_prihod),
        drzava_odhod: cleanText(offer.drzava_odhod),
        drzava_prihod: cleanText(offer.drzava_prihod),
        posta_odhod: cleanText(offer.posta_odhod),
        posta_prihod: cleanText(offer.posta_prihod),

        datum: cleanText(offer.datum),
        loading_date: cleanText(offer.loading_date),
        delivery_date: cleanText(offer.delivery_date),
        delivery_date_fixed: cleanText(offer.delivery_date_fixed),

        weight: cleanText(offer.weight),
        volume: cleanText(offer.volume),
        dimensions: cleanText(offer.dimensions),
        loading_count: cleanText(offer.loading_count),
        unloading_count: cleanText(offer.unloading_count),

        truck_required: cleanText(offer.truck_required),
        trailer_required: cleanText(offer.trailer_required),
        small_truck: cleanText(offer.small_truck),
        vehicle_type_id: cleanText(offer.vehicle_type_id),
        other_setup: cleanText(offer.other_setup),

        adr: cleanText(offer.adr),
        adr_note: cleanText(offer.adr_note),
        remarks: cleanText(offer.remarks),

        contact_name: cleanText(offer.contact_name),
        contact_phone: cleanText(offer.contact_phone),
        contact_fax: cleanText(offer.contact_fax),

        updated_at: new Date().toISOString()
    };
}

module.exports = async (req, res) => {
    try {
        const offers = await fetchCargoOffers();

        if (offers.length === 0) {
            return res.status(200).json({
                success: true,
                count: 0,
                message: 'CargoTrans trenutno ni vrnil nobene ponudbe. Obstoječi podatki so ohranjeni.'
            });
        }

        const records = offers.map(mapOffer);

        const missingIds = records.filter(record => !record.tovor_id);

        if (missingIds.length > 0) {
            throw new Error(
                'Uvoz ustavljen: ' + missingIds.length +
                ' ponudb nima tovor_id.'
            );
        }

        const uniqueIds = new Set(records.map(record => record.tovor_id));

        if (uniqueIds.size !== records.length) {
            throw new Error('Uvoz ustavljen: JSON vsebuje podvojene tovor_id.');
        }

        const { data, error } = await supabase
            .from('cargotrans_offers')
            .upsert(records, {
                onConflict: 'tovor_id'
            })
            .select('tovor_id');

        if (error) {
            throw new Error('Supabase napaka: ' + error.message);
        }

        return res.status(200).json({
            success: true,
            received: offers.length,
            imported: data ? data.length : records.length,
            message: 'CargoTrans ponudbe so uspešno sinhronizirane.'
        });

    } catch (error) {
        console.error('CargoTrans sync error:', error);

        return res.status(500).json({
            success: false,
            error: error.message
        });
    }
};

