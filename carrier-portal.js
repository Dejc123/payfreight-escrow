/**
 * @file carrier-portal.js
 * @copyright © 2026 PayFreight.io. All rights reserved.
 * @description PayFreight.io Protocol - Decentralized Escrow Carrier Portal & Notification Handler.
 * Modular component for carrier deposit, shipment verification, and status updates.
 */

class PayFreightCarrierPortal {
    constructor(options = {}) {
        this.protocolVersion = "1.0.0-payfreight.io";
        this.containerId = options.containerId || "payfreight-carrier-root";
    }

    /**
     * Initialize and render the carrier portal form with pre-filled parameters.
     * @param {Object} data - Pre-filled data from magic link (shipperId, cargoId, shipperEmail)
     */
    renderForm(data = {}) {
        const { shipperId = '', cargoId = '', shipperEmail = '' } = data;

        return `
            <!-- PayFreight.io Protocol - Carrier Portal Component -->
            <div class="payfreight-portal-wrapper" style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; border: 1px solid #e1e4e8; border-radius: 8px; background-color: #f6f8fa;">
                <div style="text-align: right; font-size: 11px; color: #586069; margin-bottom: 10px;">
                    © 2026 PayFreight.io Protocol. All rights reserved.
                </div>
                
                <h2 style="color: #24292e; border-bottom: 2px solid #0366d6; padding-bottom: 8px; margin-top: 0;">
                    Carrier Portal & Deposit Execution
                </h2>
                <p style="font-size: 14px; color: #586069;">
                    Review shipment details securely via decentralized smart contract protocol. Neutral arbitration enabled.
                </p>

                <form id="payfreightCarrierForm" onsubmit="PayFreightCarrierPortal.handleAction(event)">
                    <div style="margin-bottom: 15px;">
                        <label style="display: block; font-weight: bold; margin-bottom: 5px; font-size: 13px;">Shipper ID:</label>
                        <input type="text" name="shipperId" value="${shipperId}" readonly style="width: 100%; padding: 8px; border: 1px solid #d1d5da; border-radius: 4px; background-color: #e1e4e8;" />
                    </div>

                    <div style="margin-bottom: 15px;">
                        <label style="display: block; font-weight: bold; margin-bottom: 5px; font-size: 13px;">Cargo ID / Shipment ID:</label>
                        <input type="text" name="cargoId" value="${cargoId}" readonly style="width: 100%; padding: 8px; border: 1px solid #d1d5da; border-radius: 4px; background-color: #e1e4e8;" />
                    </div>

                    <div style="margin-bottom: 15px;">
                        <label style="display: block; font-weight: bold; margin-bottom: 5px; font-size: 13px;">Shipper Email:</label>
                        <input type="email" name="shipperEmail" value="${shipperEmail}" readonly style="width: 100%; padding: 8px; border: 1px solid #d1d5da; border-radius: 4px; background-color: #e1e4e8;" />
                    </div>

                    <div style="display: flex; gap: 10px; margin-top: 20px;">
                        <button type="button" onclick="PayFreightCarrierPortal.submitAction('accept')" style="flex: 1; background-color: #28a745; color: white; border: none; padding: 10px; border-radius: 4px; font-weight: bold; cursor: pointer;">
                            Accept & Pay Deposit
                        </button>
                        <button type="button" onclick="PayFreightCarrierPortal.submitAction('cancel')" style="flex: 1; background-color: #d73a49; color: white; border: none; padding: 10px; border-radius: 4px; font-weight: bold; cursor: pointer;">
                            Cancel / Mutual Abort
                        </button>
                    </div>
                </form>
            </div>
        `;
    }

    /**
     * Handle actions triggered by the carrier (Accept/Deposit or Cancel).
     * @param {string} actionType - Action type ('accept' or 'cancel')
     */
    static submitAction(actionType) {
        if (actionType === 'accept') {
            alert("PayFreight Protocol: Processing carrier deposit via smart contract...");
            // TODO: Integrate web3 wallet connection / smart contract call here
        } else if (actionType === 'cancel') {
            alert("PayFreight Protocol: Initiating mutual cancellation workflow.");
            // TODO: Handle cancellation request and trigger email notifications
        }
    }
}

// Export module for Node.js / Server or Browser use
if (typeof module !== 'undefined' && module.exports) {
    module.exports = PayFreightCarrierPortal;
}
