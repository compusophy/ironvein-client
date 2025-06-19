import init, { IronVeinClient } from './pkg/client.js';

let gameClient = null;
let connected = false;
let inBattle = false;
let pingStartTime = 0;
let pingInterval = null;
let onlinePlayers = new Map();
let consecutiveSuccesses = 0; // Ping stability tracking

// Two-stage connection: Connect → Join Battle
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
        if (e.key === 'Enter') connectToServer();
    });
    
    document.getElementById('roomInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            if (connected && !inBattle) {
                joinBattle();
            } else {
                connectToServer();
            }
        }
    });
    
    document.getElementById('chatInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') sendMessage();
    });
    
    // Expose gameClient globally for Rust callbacks
    window.gameClient = gameClient;
}

// Stage 1: Connect to server (see lobby, chat, players)
window.connectToServer = async function() {
    const username = document.getElementById('usernameInput').value.trim();
    const room = document.getElementById('roomInput').value.trim();
    
    if (!username) {
        alert('Please enter your warrior name!');
        return;
    }
    
    if (!room) {
        alert('Please enter battlefield name!');
        return;
    }
    
    try {
        // Set user info and connect to server
        gameClient.set_user_info(username, room);
        await gameClient.connect_to_server(); // Just connect, don't join game yet
        
        connected = true;
        updateUI();
        updateUserDisplay(username, room);
        startPingSystem();
        
        appendSystemMessage('🌐 Connected to server! You can now see chat and players.');
        appendSystemMessage('🎮 Click "Join Battle" to spawn your unit and start playing.');
        
    } catch (error) {
        console.error('Connection failed:', error);
        appendSystemMessage(`❌ Connection failed: ${error}`);
    }
};

// Stage 2: Join the actual battle (spawn player)
window.joinBattle = async function() {
    if (!connected) {
        appendSystemMessage('❌ Connect to server first!');
        return;
    }
    
    if (inBattle) {
        appendSystemMessage('❌ Already in battle!');
        return;
    }
    
    try {
        await gameClient.setup_game_canvas('gameCanvas');
        await gameClient.join_battle(); // Spawn player in game
        
        inBattle = true;
        updateUI();
        
        appendSystemMessage('⚔️ Joined battle! Click on the grid to move your unit.');
        
    } catch (error) {
        console.error('Failed to join battle:', error);
        appendSystemMessage(`❌ Failed to join battle: ${error}`);
    }
};

// Send chat message (works in lobby and battle)
window.sendMessage = function() {
    if (!connected) {
        appendSystemMessage('❌ Not connected to server');
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

// UI updates
function updateUI() {
    const setupPanel = document.getElementById('setupPanel');
    const chatInput = document.getElementById('chatInput');
    const connectBtn = document.getElementById('connectBtn');
    const joinBtn = document.getElementById('joinBtn');
    
    if (inBattle) {
        // In battle: hide setup, enable chat
        setupPanel.classList.add('hidden');
        chatInput.disabled = false;
        chatInput.placeholder = 'Type your battle message...';
    } else if (connected) {
        // Connected but not in battle: show lobby
        if (connectBtn) {
            connectBtn.textContent = '🌐 Connected to Server';
            connectBtn.disabled = true;
        }
        if (joinBtn) {
            joinBtn.disabled = false;
            joinBtn.style.opacity = '1';
        }
        chatInput.disabled = false;
        chatInput.placeholder = 'Chat in lobby...';
    } else {
        // Not connected: show connect button
        setupPanel.classList.remove('hidden');
        if (connectBtn) {
            connectBtn.textContent = '🌐 Connect to Server';
            connectBtn.disabled = false;
        }
        if (joinBtn) {
            joinBtn.disabled = true;
            joinBtn.style.opacity = '0.5';
        }
        chatInput.disabled = true;
        chatInput.placeholder = 'Connect to chat...';
    }
}

function updateUserDisplay(username, room) {
    document.getElementById('userDisplay').textContent = username;
    document.getElementById('roomDisplay').textContent = `Room: ${room}`;
}

// Optimized ping system - world-class implementation
function startPingSystem() {
    if (pingInterval) clearInterval(pingInterval);
    
    let pingCount = 0;
    
    // Adaptive ping frequency based on connection stability
    const getPingInterval = () => {
        if (consecutiveSuccesses > 10) return 5000; // 5s when stable
        if (consecutiveSuccesses > 5) return 3000;  // 3s when fairly stable
        return 2000; // 2s when unstable or starting
    };
    
    const doPing = () => {
        if (connected && gameClient) {
            try {
                pingStartTime = performance.now();
                pingCount++;
                
                // Use minimal payload for efficiency
                gameClient.send_ping(); // Will implement this as separate method
                
                // Schedule next ping with adaptive interval
                setTimeout(doPing, getPingInterval());
                
            } catch (error) {
                console.error('❌ Ping failed:', error);
                consecutiveSuccesses = 0;
                connected = false;
                updateUI();
                updatePingDisplay('DISCONNECTED');
            }
        }
    };
    
    // Start immediate ping
    doPing();
    
    console.log('📶 Adaptive ping system started');
}

// Ping response handler - called from Rust
window.onPingReceived = function() {
    if (pingStartTime > 0) {
        const ping = Math.round(performance.now() - pingStartTime);
        updatePingDisplay(ping);
        pingStartTime = 0;
        
        // Track stability for adaptive pinging
        if (ping < 200) {
            consecutiveSuccesses = Math.min(consecutiveSuccesses + 1, 15);
        } else {
            consecutiveSuccesses = Math.max(consecutiveSuccesses - 1, 0);
        }
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
    
    // Update ping color - world-class thresholds
    pingDisplay.classList.remove('ping-good', 'ping-ok', 'ping-bad');
    if (ping < 30) {
        pingDisplay.classList.add('ping-good'); // Excellent
    } else if (ping < 80) {
        pingDisplay.classList.add('ping-ok'); // Good
    } else {
        pingDisplay.classList.add('ping-bad'); // Poor
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
    messageDiv.style.color = '#4ecdc4';
    messageDiv.style.fontStyle = 'italic';
    
    chatMessages.appendChild(messageDiv);
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

// Start the lightweight interface
run().catch(console.error); 