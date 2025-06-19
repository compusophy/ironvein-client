import init, { IronVeinClient } from './pkg/client.js';

let gameClient = null;
let connected = false;
let pingStartTime = 0;
let pingInterval = null;
let onlinePlayers = new Map();

// Minimal JavaScript - just UI interface, everything else in Rust
async function run() {
    await init();
    console.log('🚀 Lightweight JS interface loaded');
    
    gameClient = new IronVeinClient();
    setupEventListeners();
    updateUI();
}

function setupEventListeners() {
    // Enter key handlers
    document.getElementById('usernameInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') connectToGame();
    });
    
    document.getElementById('roomInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') connectToGame();
    });
    
    document.getElementById('chatInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') sendMessage();
    });
    
    // Expose gameClient globally for Rust callbacks
    window.gameClient = gameClient;
}

// Connect to game - minimal JS, let Rust handle everything
window.connectToGame = async function() {
    const username = document.getElementById('usernameInput').value.trim();
    const room = document.getElementById('roomInput').value.trim();
    
    if (!username || !room) {
        alert('Please enter both battle name and battlefield!');
        return;
    }
    
    try {
        // Let Rust handle everything
        gameClient.set_user_info(username, room);
        await gameClient.setup_game_canvas('gameCanvas');
        await gameClient.connect();
        
        connected = true;
        updateUI();
        updateUserDisplay(username, room);
        startPingSystem();
        
        console.log('🎮 Connected! Rust is handling all game logic.');
        
    } catch (error) {
        console.error('Connection failed:', error);
        appendSystemMessage(`❌ Connection failed: ${error}`);
    }
};

// Send chat message - let Rust handle pending state
window.sendMessage = function() {
    if (!connected || !gameClient) {
        appendSystemMessage('❌ Not connected');
        return;
    }
    
    const chatInput = document.getElementById('chatInput');
    const message = chatInput.value.trim();
    
    if (!message) return;
    
    try {
        gameClient.send_message(message);
        chatInput.value = '';
    } catch (error) {
        console.error('Failed to send message:', error);
        appendSystemMessage(`❌ Failed to send: ${error}`);
    }
};

// Simple UI updates - keep JS minimal
function updateUI() {
    const setupPanel = document.getElementById('setupPanel');
    const chatInput = document.getElementById('chatInput');
    
    if (connected) {
        setupPanel.classList.add('hidden');
        chatInput.disabled = false;
        chatInput.placeholder = 'Type your message...';
    } else {
        setupPanel.classList.remove('hidden');
        chatInput.disabled = true;
        chatInput.placeholder = 'Connect to chat...';
    }
}

function updateUserDisplay(username, room) {
    document.getElementById('userDisplay').textContent = username;
    document.getElementById('roomDisplay').textContent = `Room: ${room}`;
}

// Simple ping system - 2 second intervals
function startPingSystem() {
    if (pingInterval) clearInterval(pingInterval);
    
    pingInterval = setInterval(() => {
        if (connected && gameClient) {
            // Check if WebSocket is actually connected before sending ping
            try {
                pingStartTime = performance.now();
                gameClient.send_message('__ping__');
            } catch (error) {
                console.error('❌ Ping failed, connection lost:', error);
                // Stop pinging if connection is lost
                if (pingInterval) {
                    clearInterval(pingInterval);
                    pingInterval = null;
                }
                connected = false;
                updateUI();
                updatePingDisplay('DISCONNECTED');
            }
        }
    }, 2000);
}

// Ping response handler - called from Rust
window.onPingReceived = function() {
    if (pingStartTime > 0) {
        const ping = Math.round(performance.now() - pingStartTime);
        updatePingDisplay(ping);
        pingStartTime = 0;
    }
}

function updatePingDisplay(ping) {
    const pingDisplay = document.getElementById('pingDisplay');
    if (!pingDisplay) return;
    
    if (ping === 'DISCONNECTED') {
        pingDisplay.textContent = 'Ping: DISCONNECTED';
        pingDisplay.classList.remove('ping-good', 'ping-ok', 'ping-bad');
        pingDisplay.classList.add('ping-bad');
        return;
    }
    
    pingDisplay.textContent = `Ping: ${ping}ms`;
    
    // Update ping color
    pingDisplay.classList.remove('ping-good', 'ping-ok', 'ping-bad');
    if (ping < 50) {
        pingDisplay.classList.add('ping-good');
    } else if (ping < 150) {
        pingDisplay.classList.add('ping-ok');
    } else {
        pingDisplay.classList.add('ping-bad');
    }
}

// Player list management - called from Rust
window.updatePlayerInList = function(username, x, y, health, resources) {
    onlinePlayers.set(username, { username, x, y, health, resources });
    updateOnlinePlayersList();
};

window.removePlayerFromList = function(username) {
    onlinePlayers.delete(username);
    updateOnlinePlayersList();
};

function updateOnlinePlayersList() {
    const playersList = document.getElementById('playersList');
    playersList.innerHTML = '';
    
    if (onlinePlayers.size === 0) {
        const noPlayersDiv = document.createElement('div');
        noPlayersDiv.className = 'player-item';
        noPlayersDiv.innerHTML = '<span class="player-name">No players online</span>';
        playersList.appendChild(noPlayersDiv);
        return;
    }
    
    // Sort players alphabetically
    const sortedPlayers = Array.from(onlinePlayers.values()).sort((a, b) => a.username.localeCompare(b.username));
    const myUsername = document.getElementById('userDisplay').textContent;
    
    sortedPlayers.forEach(player => {
        const playerDiv = document.createElement('div');
        playerDiv.className = 'player-item';
        
        const isMe = player.username === myUsername;
        const nameStyle = isMe ? 'color: var(--iron-accent); font-weight: bold;' : '';
        
        playerDiv.innerHTML = `
            <span class="player-name" style="${nameStyle}">${player.username}${isMe ? ' (YOU)' : ''}</span>
            <span class="player-pos">(${player.x}, ${player.y})</span>
        `;
        
        playersList.appendChild(playerDiv);
    });
}

// Simple system message helper
function appendSystemMessage(message) {
    const chatMessages = document.getElementById('chatMessages');
    const messageDiv = document.createElement('div');
    messageDiv.textContent = message;
    messageDiv.style.color = '#ff6b6b';
    messageDiv.style.fontStyle = 'italic';
    
    chatMessages.appendChild(messageDiv);
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

// Start the lightweight interface
run().catch(console.error); 