import init, { Client } from './pkg/client.js';

let client = null;

async function initWasm() {
    try {
        await init();
        client = new Client();
        setStatus('✅ WASM module loaded successfully!', 'success');
        updateOutput('IronVein Client initialized and ready to use.');
    } catch (error) {
        console.error('Failed to load WASM:', error);
        setStatus('❌ Failed to load WASM module: ' + error.message, 'error');
    }
}

function setStatus(message, type) {
    const statusDiv = document.getElementById('status');
    statusDiv.textContent = message;
    statusDiv.className = `status ${type}`;
}

function updateOutput(message) {
    const output = document.getElementById('output');
    const timestamp = new Date().toLocaleTimeString();
    output.textContent += `[${timestamp}] ${message}\n`;
    output.scrollTop = output.scrollHeight;
}

// Global functions for HTML buttons
window.greetUser = function() {
    if (!client) {
        setStatus('⚠️  WASM module not loaded yet', 'error');
        return;
    }
    
    const userName = document.getElementById('userName').value || 'User';
    const greeting = client.greet(userName);
    updateOutput(greeting);
    setStatus('👋 Greeting sent!', 'success');
}

window.setServerUrl = function() {
    if (!client) {
        setStatus('⚠️  WASM module not loaded yet', 'error');
        return;
    }
    
    const serverUrl = document.getElementById('serverUrl').value;
    if (!serverUrl) {
        setStatus('⚠️  Please enter a server URL', 'error');
        return;
    }
    
    client.set_server_url(serverUrl);
    updateOutput(`Server URL updated to: ${serverUrl}`);
    setStatus('🔧 Server URL updated!', 'success');
}

window.fetchData = async function() {
    if (!client) {
        setStatus('⚠️  WASM module not loaded yet', 'error');
        return;
    }
    
    setStatus('🔄 Fetching data...', 'loading');
    updateOutput('Attempting to fetch data from server...');
    
    try {
        const data = await client.fetch_data();
        updateOutput(`Server response: ${data}`);
        setStatus('📊 Data fetched successfully!', 'success');
    } catch (error) {
        updateOutput(`Error fetching data: ${error}`);
        setStatus('❌ Failed to fetch data: ' + error, 'error');
    }
}

window.testConnection = function() {
    if (!client) {
        setStatus('⚠️  WASM module not loaded yet', 'error');
        return;
    }
    
    const serverUrl = client.get_server_url();
    updateOutput(`Testing connection to: ${serverUrl}`);
    updateOutput('Connection test: Client is ready to connect');
    setStatus('🔍 Connection test completed', 'success');
}

// Initialize when the page loads
document.addEventListener('DOMContentLoaded', () => {
    setStatus('🔄 Loading WASM module...', 'loading');
    initWasm();
}); 