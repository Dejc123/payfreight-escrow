/**
 * ============================================================================
 * PAYFREIGHT PROTOCOL – BACKEND MAILER & NOTIFICATION ENGINE
 * ============================================================================
 * Handles automated transactional emails, tokenized confirmation links,
 * and escrow deposit alerts for Shippers and Carriers (US & EU compliance).
 * ============================================================================
 */

const nodemailer = require('nodemailer'); // Ali uporabljen izbran mail transport API

class PayFreightMailer {
    constructor(config) {
        this.transporter = nodemailer.createTransport({
            host: config.host || 'smtp.payfreight.io',
            port: config.port || 465,
            secure: true,
            auth: {
                user: config.user,
                pass: config.pass
            }
        });
        this.baseUrl = config.baseUrl || 'https://app.payfreight.io';
    }

    // 1. Shipper vnese fure -> Poziv prevozniku za vplačilo €250/$250 kavcije
    async sendCarrierFeeRequest(carrierEmail, orderId, loadId, amount, secureToken) {
        const verifyLink = `${this.baseUrl}/verify-fee?token=${secureToken}&order=${orderId}`;
        const mailOptions = {
            from: '"PayFreight Protocol Escrow" <escrow@payfreight.io>',
            to: carrierEmail,
            subject: `[Action Required] Secure Freight Order ${orderId} – €250/$250 Commitment Deposit`,
            html: `
                <div style="font-family: sans-serif; font-size: 14px; color: #27221d; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #e8e2d8; border-radius: 12px;">
                    <h2 style="color: #0284c7; margin-top: 0;">PayFreight Secure Settlement Notice</h2>
                    <p>Hello Carrier,</p>
                    <p>A new freight transport order (<strong>Order ID: ${orderId}</strong>, Load: ${loadId}) has been assigned to you with a freight price of <strong>${amount}</strong>.</p>
                    <p>To lock this assignment and activate institutional escrow protection, please secure your mutual commitment deposit (€250 / $250):</p>
                    <div style="text-align: center; margin: 25px 0;">
                        <a href="${verifyLink}" style="background-color: #df9b41; color: #ffffff; padding: 12px 24px; text-decoration: none; font-weight: bold; border-radius: 8px; display: inline-block;">Pay €250 / $250 Commitment Fee ➔</a>
                    </div>
                    <p style="font-size: 12px; color: #786f66;">Once paid, you will receive your secure upload link for the e-CMR / POD delivery document.</p>
                </div>
            `
        };
        return await this.transporter.sendMail(mailOptions);
    }

    // 2. Prevoznik vplača kavcijo -> Potrditev prevozniku in link za e-CMR
    async sendCarrierFeeConfirmation(carrierEmail, orderId, uploadToken) {
        const uploadLink = `${this.baseUrl}/upload-cmr?token=${uploadToken}&order=${orderId}`;
        const mailOptions = {
            from: '"PayFreight Protocol Escrow" <escrow@payfreight.io>',
            to: carrierEmail,
            subject: `[Confirmed] Commitment Fee Received – Order ${orderId}`,
            html: `
                <div style="font-family: sans-serif; font-size: 14px; color: #27221d; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #bbf7d0; border-radius: 12px; background-color: #f0fdf4;">
                    <h2 style="color: #15803d; margin-top: 0;">✅ Commitment Fee Successfully Secured</h2>
                    <p>Your €250 / $250 commitment deposit for Order <strong>${orderId}</strong> is verified in the escrow vault.</p>
                    <p>Keep this secure direct link handy for when your transport is completed to upload your e-CMR / POD:</p>
                    <div style="text-align: center; margin: 25px 0;">
                        <a href="${uploadLink}" style="background-color: #16a34a; color: #ffffff; padding: 12px 24px; text-decoration: none; font-weight: bold; border-radius: 8px; display: inline-block;">Upload e-CMR / POD for Payout ➔</a>
                    </div>
                </div>
            `
        };
        return await this.transporter.sendMail(mailOptions);
    }

    // 3. Shipper vplača glavni znesek -> Obvestilo o varnem depozitu v escrowu
    async sendEscrowFundedNotice(recipientEmail, orderId, amount, role) {
        const mailOptions = {
            from: '"PayFreight Protocol Escrow" <escrow@payfreight.io>',
            to: recipientEmail,
            subject: `[Escrow Secured] Full Freight Funds Locked for Order ${orderId}`,
            html: `
                <div style="font-family: sans-serif; font-size: 14px; color: #27221d; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #ebdccb; border-radius: 12px;">
                    <h3 style="color: #0f172a;">🔒 Institutional Escrow Vault Funded</h3>
                    <p>The total freight amount of <strong>${amount}</strong> for Order <strong>${orderId}</strong> has been successfully deposited and locked under 1:1 fiat-backed security.</p>
                    <p>Role registered: <strong>${role}</strong>. Funds remain fully protected until delivery verification or mutual agreement.</p>
                </div>
            `
        };
        return await this.transporter.sendMail(mailOptions);
    }

    // 4. Sprožitev Mutual Cancellation -> Obveščanje nasprotne stranke s potrditvenim linkom
    async sendMutualCancelRequest(counterpartEmail, orderId, cancelToken) {
        const confirmLink = `${this.baseUrl}/mutual-cancel?token=${cancelToken}&order=${orderId}`;
        const mailOptions = {
            from: '"PayFreight Protocol Mediation" <disputes@payfreight.io>',
            to: counterpartEmail,
            subject: `[Action Required] Mutual Cancellation Request for Order ${orderId}`,
            html: `
                <div style="font-family: sans-serif; font-size: 14px; color: #27221d; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #e8e2d8; border-radius: 12px;">
                    <h3 style="color: #b87b28;">🤝 Mutual Cancellation Proposed</h3>
                    <p>The counterpart has requested a mutual cancellation for transport order <strong>${orderId}</strong>.</p>
                    <p>If you approve, <strong>100% of the freight principal will be returned instantly to the Shipper</strong>, and both commitment deposits (€250/$250) will be fully refunded with zero penalties.</p>
                    <div style="text-align: center; margin: 25px 0;">
                        <a href="${confirmLink}" style="background-color: #27221d; color: #ffffff; padding: 12px 24px; text-decoration: none; font-weight: bold; border-radius: 8px; display: inline-block;">Approve Mutual Cancellation ➔</a>
                    </div>
                </div>
            `
        };
        return await this.transporter.sendMail(mailOptions);
    }

    // 5. Sprožitev spora / Fault Attribution ("Rdeča zadeva")
    async sendDisputeNotice(counterpartEmail, orderId, faultReason, disputeToken) {
        const disputeLink = `${this.baseUrl}/dispute-response?token=${disputeToken}&order=${orderId}`;
        const mailOptions = {
            from: '"PayFreight Protocol Mediation" <disputes@payfreight.io>',
            to: counterpartEmail,
            subject: `[URGENT] Dispute / Fault Claim Filed for Order ${orderId}`,
            html: `
                <div style="font-family: sans-serif; font-size: 14px; color: #27221d; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #fecaca; border-radius: 12px; background-color: #fef2f2;">
                    <h3 style="color: #dc2626;">⚖️ Protocol Dispute Notice</h3>
                    <p>A formal claim or default has been submitted regarding loading-day performance on Order <strong>${orderId}</strong>.</p>
                    <p><strong>Stated Reason / Fault:</strong> ${faultReason}</p>
                    <p>Please review and submit your counter-statement or evidence within the protocol timeframe:</p>
                    <div style="text-align: center; margin: 25px 0;">
                        <a href="${disputeLink}" style="background-color: #dc2626; color: #ffffff; padding: 12px 24px; text-decoration: none; font-weight: bold; border-radius: 8px; display: inline-block;">Review Dispute & Respond ➔</a>
                    </div>
                </div>
            `
        };
        return await this.transporter.sendMail(mailOptions);
    }
}

module.exports = PayFreightMailer;
