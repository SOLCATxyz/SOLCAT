document.addEventListener('DOMContentLoaded', () => {
  const addressInput = document.getElementById('addressInput');
  const checkButton = document.getElementById('checkAddress');
  const addressStatus = document.getElementById('addressStatus');
  const statusText = document.getElementById('statusText');
  const riskLevel = document.getElementById('riskLevel');
  const reportButton = document.getElementById('reportAddress');
  const viewDashboardButton = document.getElementById('viewDashboard');

  // Check address when button is clicked
  checkButton.addEventListener('click', async () => {
    const address = addressInput.value.trim();
    if (!isValidSolanaAddress(address)) {
      showError('Invalid Solana address');
      return;
    }

    try {
      const status = await checkAddressStatus(address);
      showAddressStatus(status);
    } catch (error) {
      showError('Error checking address status');
    }
  });

  // Report suspicious address
  reportButton.addEventListener('click', async () => {
    const address = addressInput.value.trim();
    if (!isValidSolanaAddress(address)) {
      showError('Invalid Solana address');
      return;
    }

    try {
      await reportSuspiciousAddress(address);
      showSuccess('Address reported successfully');
    } catch (error) {
      showError('Error reporting address');
    }
  });

  // View dashboard
  viewDashboardButton.addEventListener('click', () => {
    chrome.tabs.create({ url: 'https://www.solcat.work/dashboard' });
  });

  // Helper functions
  function isValidSolanaAddress(address) {
    return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
  }

  async function checkAddressStatus(address) {
    const response = await fetch(`https://api.solcat.work/check/${address}`);
    if (!response.ok) {
      throw new Error('Network response was not ok');
    }
    return response.json();
  }

  async function reportSuspiciousAddress(address) {
    const response = await fetch('https://api.solcat.work/report', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ address }),
    });
    if (!response.ok) {
      throw new Error('Network response was not ok');
    }
    return response.json();
  }

  function showAddressStatus(status) {
    addressStatus.classList.remove('hidden');
    reportButton.classList.remove('hidden');
    
    statusText.textContent = status.description;
    riskLevel.textContent = `Risk Level: ${status.risk}`;
    riskLevel.className = `risk-${status.risk.toLowerCase()}`;
  }

  function showError(message) {
    addressStatus.classList.remove('hidden');
    reportButton.classList.add('hidden');
    statusText.textContent = message;
    riskLevel.textContent = '';
  }

  function showSuccess(message) {
    addressStatus.classList.remove('hidden');
    reportButton.classList.add('hidden');
    statusText.textContent = message;
    riskLevel.textContent = '';
  }
}); 