const nodemailer = require('nodemailer');

// General SMTP email server configuration
const transporter = nodemailer.createTransport({
  host: 'smtp.yourmailserver.com', // Enter your real SMTP server here
  port: 587,
  secure: false,
  auth: {
    user: 'noreply@cargotrans.io',
    pass: 'your_email_password'
  }
});

// General function to send emails
async function sendEmail(to, subject, htmlContent) {
  try {
    let info = await transporter.sendMail({
      from: '"CargoTrans Protocol" <noreply@cargotrans.io>',
      to: to,
      subject: subject,
      html: htmlContent
    });
    console.log("Email successfully sent: %s", info.messageId);
    return true;
  } catch (error) {
    console.error("Error sending email:", error);
    return false;
  }
}

module.exports = { sendEmail };
