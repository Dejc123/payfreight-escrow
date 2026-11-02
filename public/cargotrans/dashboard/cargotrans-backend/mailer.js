const nodemailer = require('nodemailer');

// Skupna nastavitev e-poštnega strežnika (SMTP)
const transporter = nodemailer.createTransport({
  host: 'smtp.yourmailserver.com', // Tukaj vpišeš svoj pravi SMTP strežnik
  port: 587,
  secure: false,
  auth: {
    user: 'noreply@cargotrans.io',
    pass: 'your_email_password'
  }
});

// Splošna funkcija za pošiljanje mailov
async function sendEmail(to, subject, htmlContent) {
  try {
    let info = await transporter.sendMail({
      from: '"CargoTrans Protocol" <noreply@cargotrans.io>',
      to: to,
      subject: subject,
      html: htmlContent
    });
    console.log("Email uspešno poslan: %s", info.messageId);
    return true;
  } catch (error) {
    console.error("Napaka pri pošiljanju emaila:", error);
    return false;
  }
}

module.exports = { sendEmail };
