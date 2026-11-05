const https = require('https');

// Funkcija za pridobivanje tovornih ponudb iz starega ASP sistema
function fetchCargoOffers(callback) {
    const url = 'https://www.cargotrans.net/export_offers.asp';

    https.get(url, (res) => {
        let data = '';

        // Zbiramo podatke medtem ko pritekajo
        res.on('data', (chunk) => {
            data += chunk;
        });

        // Ko je prenos končan
        res.on('end', () => {
            try {
                // Pretvorimo prejeti JSON niz v objekt/tabelo v JavaScriptu
                const offers = JSON.parse(data);
                callback(null, offers);
            } catch (e) {
                console.error('Napaka pri parsanju JSON podatkov iz ASP-ja:', e.message);
                callback(e, null);
            }
        });
    }).on('error', (err) => {
        console.error('Napaka pri povezovanju s cargotrans.net:', err.message);
        callback(err, null);
    });
}

// Izvozimo funkcijo, da jo lahko uporabite v server.js ali drugje
module.exports = { fetchCargoOffers };
