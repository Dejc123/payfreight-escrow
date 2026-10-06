/**
 * Opens the Escrow Modal overlay
 */
function openEscrowModal() {
  document.getElementById('escrowModal').classList.remove('hidden');
}

/**
 * Closes the Escrow Modal overlay
 */
function closeEscrowModal() {
  document.getElementById('escrowModal').classList.add('hidden');
}

/**
 * Handles Tab navigation switching (PDF, Exchange, Manual)
 * @param {string} tabName - Selected tab identifier ('pdf', 'exchange', 'manual')
 */
function switchTab(tabName) {
  // Hide all tab content containers
  document.querySelectorAll('.tab-content').forEach(el => el.classList.add('hidden'));
  
  // Reset tab button styling
  ['pdf', 'exchange', 'manual'].forEach(t => {
    const btn = document.getElementById(`tab-${t}-btn`);
    if (btn) {
      btn.className = "flex-1 py-2.5 px-3 rounded-xl border border-transparent text-gray-600 hover:bg-gray-100 flex items-center justify-center gap-2";
    }
  });

  // Display selected tab content and highlight active tab button
  document.getElementById(`tab-${tabName}`).classList.remove('hidden');
  const activeBtn = document.getElementById(`tab-${tabName}-btn`);
  if (activeBtn) {
    activeBtn.className = "flex-1 py-2.5 px-3 rounded-xl border border-amber-500 bg-white text-amber-700 shadow-sm flex items-center justify-center gap-2";
  }
}

/**
 * Handles PDF File Upload
 */
function handlePdfUpload(input) {
  if (input.files && input.files[0]) {
    const fileName = input.files[0].name;
    console.log(`[OCR/AI Parser] Uploaded file: ${fileName}`);
  }
}

/**
 * Submits payload data to the Settlement Protocol Backend
 */
async function submitEscrow() {
  const activeTab = document.querySelector('.tab-content:not(.hidden)').id;
  const paymentMethod = document.querySelector('input[name="paymentType"]:checked').value;

  let payload = {
    payment_method: paymentMethod,
    source_type: activeTab
  };

  if (activeTab === 'tab-exchange') {
    payload.exchange_details = {
      exchange_name: document.getElementById('exchangeName').value,
      cargo_id: document.getElementById('exchangeCargoId').value
    };
  } else if (activeTab === 'tab-manual') {
    payload.freight_details = {
      cargo_id: document.getElementById('manualCargoId').value,
      amount: document.getElementById('manualAmount').value,
      carrier_identifier: document.getElementById('manualCarrier').value
    };
  }

  console.log("[Escrow Protocol Payload Sent]:", payload);

  try {
    const response = await fetch('/api/create-escrow', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    
    const result = await response.json();
    console.log("[Escrow Created Successfully]:", result);
    closeEscrowModal();
  } catch (err) {
    console.error("[Backend Settlement API Error]:", err);
  }
}
