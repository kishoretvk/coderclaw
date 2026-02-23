// TitanClaw Setup Wizard JavaScript

// State
let currentStep = 1;
let selectedDatabase = 'libsql';
let selectedProvider = 'ollama';
let models = [];

// Initialize
document.addEventListener('DOMContentLoaded', () => {
  initCards();
  initProviderCards();
});

// Initialize database selection cards
function initCards() {
  const cards = document.querySelectorAll('[data-step="1"] .card');
  cards.forEach(card => {
    card.addEventListener('click', () => {
      cards.forEach(c => c.classList.remove('selected'));
      card.classList.add('selected');
      selectedDatabase = card.dataset.value;
      
      // Show/hide PostgreSQL URL field
      const urlGroup = document.getElementById('postgres-url-group');
      urlGroup.style.display = selectedDatabase === 'postgres' ? 'block' : 'none';
    });
  });
}

// Initialize provider selection cards
function initProviderCards() {
  const cards = document.querySelectorAll('[data-step="2"] .card');
  cards.forEach(card => {
    card.addEventListener('click', () => {
      cards.forEach(c => c.classList.remove('selected'));
      card.classList.add('selected');
      selectedProvider = card.dataset.value;
      
      // Show/hide provider-specific config
      document.querySelectorAll('.provider-fields').forEach(el => {
        el.style.display = 'none';
      });
      const providerFields = document.querySelector(`.provider-fields[data-provider="${selectedProvider}"]`);
      if (providerFields) {
        providerFields.style.display = 'block';
      }
    });
  });
}

// Navigate to next step
function nextStep(step) {
  // Validate current step
  if (step === 1) {
    // Database is always valid
  } else if (step === 2) {
    // Provider is always valid
  } else if (step === 3) {
    const modelSelect = document.getElementById('model-select');
    const modelManual = document.getElementById('model-manual');
    if (!modelSelect.value && !modelManual.value) {
      alert('Please select or enter a model');
      return;
    }
  }
  
  // Update progress
  updateProgress(step, 'active');
  if (step > 1) {
    updateProgress(step - 1, 'completed');
  }
  
  // Show/hide steps
  document.querySelectorAll('.step').forEach(el => {
    el.classList.remove('active');
  });
  document.querySelector(`.step[data-step="${step}"]`).classList.add('active');
  
  currentStep = step;
  
  // If going to review step, populate review
  if (step === 4) {
    populateReview();
  }
}

// Navigate to previous step
function prevStep(step) {
  updateProgress(step, 'active');
  updateProgress(step + 1, '');
  
  document.querySelectorAll('.step').forEach(el => {
    el.classList.remove('active');
  });
  document.querySelector(`.step[data-step="${step}"]`).classList.add('active');
  
  currentStep = step;
}

// Update progress bar
function updateProgress(step, status) {
  const progressStep = document.querySelector(`.progress-step[data-step="${step}"]`);
  if (progressStep) {
    progressStep.classList.remove('active', 'completed');
    if (status) {
      progressStep.classList.add(status);
    }
  }
}

// Fetch available models
async function fetchModels() {
  const loadingEl = document.getElementById('models-loading');
  const errorEl = document.getElementById('models-error');
  const selectEl = document.getElementById('model-select');
  const btnEl = document.getElementById('fetch-models-btn');
  
  // Get base URL based on provider
  let baseUrl = '';
  if (selectedProvider === 'ollama') {
    baseUrl = document.getElementById('ollama-url').value || 'http://localhost:11434';
  } else if (selectedProvider === 'openai_compatible') {
    baseUrl = document.getElementById('compatible-url').value || '';
  }
  
  loadingEl.classList.add('active');
  errorEl.style.display = 'none';
  btnEl.disabled = true;
  selectEl.innerHTML = '<option value="">Loading...</option>';
  
  try {
    const params = new URLSearchParams({
      provider: selectedProvider
    });
    
    if (baseUrl) {
      params.append('base_url', baseUrl);
    }
    
    const response = await fetch(`/api/setup/models?${params}`);
    const data = await response.json();
    
    loadingEl.classList.remove('active');
    btnEl.disabled = false;
    
    if (response.ok && data.models) {
      models = data.models;
      selectEl.innerHTML = '<option value="">-- Select a model --</option>';
      models.forEach(model => {
        const option = document.createElement('option');
        option.value = model.id;
        option.textContent = `${model.name} (${model.id})`;
        selectEl.appendChild(option);
      });
    } else {
      errorEl.textContent = 'Failed to fetch models. You can enter a model name manually.';
      errorEl.style.display = 'block';
      selectEl.innerHTML = '<option value="">-- Enter manually --</option>';
    }
  } catch (error) {
    loadingEl.classList.remove('active');
    btnEl.disabled = false;
    errorEl.textContent = 'Error connecting to provider. You can enter a model name manually.';
    errorEl.style.display = 'block';
    selectEl.innerHTML = '<option value="">-- Enter manually --</option>';
  }
}

// Populate review section
function populateReview() {
  // Database
  const dbText = selectedDatabase === 'libsql' ? 'SQLite (libSQL)' : 'PostgreSQL';
  document.getElementById('review-database').textContent = dbText;
  
  // Provider
  const providerNames = {
    'ollama': 'Ollama (Local)',
    'openai': 'OpenAI',
    'anthropic': 'Anthropic',
    'openai_compatible': 'OpenAI-Compatible'
  };
  document.getElementById('review-provider').textContent = providerNames[selectedProvider] || selectedProvider;
  
  // Model
  const modelSelect = document.getElementById('model-select');
  const modelManual = document.getElementById('model-manual');
  const model = modelSelect.value || modelManual.value || '-';
  document.getElementById('review-model').textContent = model;
}

// Save configuration
async function saveConfiguration() {
  const saveBtn = document.getElementById('save-btn');
  const errorEl = document.getElementById('save-error');
  
  saveBtn.disabled = true;
  saveBtn.textContent = 'Saving...';
  errorEl.style.display = 'none';
  
  // Get model
  const modelSelect = document.getElementById('model-select');
  const modelManual = document.getElementById('model-manual');
  const selectedModel = modelSelect.value || modelManual.value;
  
  // Build request
  const request = {
    database_backend: selectedDatabase,
    llm_backend: selectedProvider,
    selected_model: selectedModel,
    gateway_port: 3000,
  };
  
  // Add database-specific fields
  if (selectedDatabase === 'postgres') {
    request.database_url = document.getElementById('database-url').value;
  }
  
  // Add provider-specific fields
  if (selectedProvider === 'ollama') {
    request.ollama_base_url = document.getElementById('ollama-url').value || 'http://localhost:11434';
  } else if (selectedProvider === 'openai') {
    request.openai_api_key = document.getElementById('openai-key').value;
  } else if (selectedProvider === 'anthropic') {
    request.anthropic_api_key = document.getElementById('anthropic-key').value;
  } else if (selectedProvider === 'openai_compatible') {
    request.openai_compatible_base_url = document.getElementById('compatible-url').value;
    request.openai_compatible_api_key = document.getElementById('compatible-key').value;
  }
  
  try {
    const response = await fetch('/api/setup/save', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(request)
    });
    
    const data = await response.json();
    
    if (response.ok && data.success) {
      // Show success
      document.getElementById('final-token').textContent = data.token;
      document.getElementById('final-link').href = data.redirect_url;
      
      document.querySelectorAll('.step').forEach(el => {
        el.classList.remove('active');
      });
      document.querySelector('.step[data-step="5"]').classList.add('active');
      
      updateProgress(4, 'completed');
    } else {
      errorEl.textContent = data.error || 'Failed to save configuration';
      errorEl.style.display = 'block';
      saveBtn.disabled = false;
      saveBtn.textContent = 'Save & Continue';
    }
  } catch (error) {
    errorEl.textContent = 'Error saving configuration: ' + error.message;
    errorEl.style.display = 'block';
    saveBtn.disabled = false;
    saveBtn.textContent = 'Save & Continue';
  }
}
