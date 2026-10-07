/**
 * ============================================================================
 * PAYFREIGHT PROTOCOL – CAUTION & REMINDERS SAFEGUARD MODULE
 * ============================================================================
 * Addresses capital lock prevention, principal freight return mechanisms,
 * ghosting protections, and automated timeout triggers for US & EU corridors.
 * ============================================================================
 */

class ProtocolSafeguards {
    constructor(dbClient, mailerInstance) {
        this.db = dbClient;
        this.mailer = mailerInstance;
        this.TIMEOUT_HOURS = 24; // 24-urni odzivni rok za stornacije/sporozume
    }

    /**
     * 1. PRINCIPAL FREIGHT RETURN SAFEGUARD (Ločitev glavnine od kavcije)
     * Preprečuje blokado sredstev shipperja ob stornaciji. Glavni znesek prevoza 
     * se vrne shipperju takoj, medtem ko kavcija ostane v mediaciji.
     */
    async executeImmediatePrincipalRefund(orderId) {
        const order = await this.db.getOrderByID(orderId);
        if (!order || order.status !== 'CANCEL_PENDING') {
            throw new Error('Order not eligible for immediate principal release.');
        }

        // Takojšnje spro</h4>stitev glavnega zneska nazaj shipperju (preko ACH / SEPA / Wire)
        await this.db.transferFundsToShipper(order.shipperId, order.freightAmount);
        
        // Posodobitev statusa: Glavnica vrnjena, kavcija (€250/$250) ostane v sporu
        await this.db.updateOrderStatus(orderId, {
            principalRefunded: true,
            status: 'FUNDS_RETURNED_DISPUTE_ACTIVE'
        });

        return { success: true, message: 'Freight principal successfully returned to Shipper.' };
    }

    /**
     * 2. AUTOMATED TIMEOUT & GHOSTING PROTECTION (Timeouts & Auto-Resolution)
     * Če se nasprotna stran na poziv za stornacijo ali spor ne odzove v 24 urah,
     * protokol avtomatski razreši primer v korist aktivne stranke.
     */
    async checkAndProcessTimeouts() {
        const pendingDisputes = await this.db.getExpiredDisputes(this.TIMEOUT_HOURS);
        
        for (const dispute of pendingDisputes) {
            // Avtomatska dodelitev kavcije stranki, ki je sprožila postopek
            await this.db.transferCommitmentFee(dispute.initiatorId, dispute.commitmentFeeAmount);
            await this.db.updateOrderStatus(dispute.orderId, {
                status: 'RESOLVED_BY_TIMEOUT',
                resolvedReason: 'Counterpart unresponsiveness (Ghosting timeout)'
            });
        }
    }

    /**
     * 3. AUTOMATED REMINDERS ENGINE (Opomniki)
     * Pošiljanje opomnikov po preteku 12 ur, če stranka ni oddala kavcije ali potrditve.
     */
    async dispatchReminders() {
        const inactiveOrders = await this.db.getInactiveOrdersNeedingReminder(12);

        for (const order of inactiveOrders) {
            if (order.stage === 'AWAITING_CARRIER_FEE') {
                await this.mailer.sendCarrierFeeRequest(
                    order.carrierEmail, 
                    order.orderId, 
                    order.loadId, 
                    order.freightAmount, 
                    order.secureToken
                );
            }
        }
    }
}

module.exports = ProtocolSafeguards;
