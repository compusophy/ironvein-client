import init, { IronVeinClient } from './pkg/client.js';

let gameClient = null;
let connected = false;
let pendingMessages = new Map(); // Track pending messages to remove when server responds
let pingStartTime = 0;
let currentPing = 0;
let pingInterval = null;
let onlinePlayers = new Map(); // Track online players

async function run() {
    // Initialize the WASM module
    await init();
    console.log('🚀 IronVein Chat WASM module loaded successfully!');
    
    // Create client instance
    gameClient = new IronVeinClient();
    
    // Set up event listeners
    setupEventListeners();
    
    // Initialize UI
    updateUI();
}

function setupEventListeners() {
    // Enter key in username/room inputs
    document.getElementById('usernameInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            connectToGame();
        }
    });
    
    document.getElementById('roomInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            connectToGame();
        }
    });
    
    // Enter key in chat input
    document.getElementById('chatInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            sendMessage();
        }
    });
    
    // Auto-update position display and stats
    setInterval(updateStats, 1000);
    
    // Expose gameClient globally for WASM callbacks
    window.gameClient = gameClient;
}

window.connectToGame = async function() {
    const usernameInput = document.getElementById('usernameInput');
    const roomInput = document.getElementById('roomInput');
    const username = usernameInput.value.trim();
    const room = roomInput.value.trim();
    
    if (!username) {
        alert('Please enter a battle name!');
        usernameInput.focus();
        return;
    }
    
    if (!room) {
        alert('Please enter a battlefield name!');
        roomInput.focus();
        return;
    }
    
    try {
        // Set user info
        gameClient.set_user_info(username, room);
        
        // Setup game canvas
        await gameClient.setup_game_canvas('gameCanvas');
        
        // Connect to server
        await gameClient.connect();
        
        // Update UI
        connected = true;
        updateUI();
        updateUserDisplay(username, room);
        
        // Start automatic ping system
        startPingSystem();
        
        // Initialize online players list
        updateOnlinePlayersList();
        
        // Show success message
        appendChatMessage('🎮 Connected to IronVein MMO RTS! Click on the grid to move your unit.');
        appendChatMessage('💬 Use the chat to coordinate with other players.');
        
        // Initial render
        setTimeout(() => {
            if (gameClient) {
                gameClient.render_game();
            }
        }, 1000);
        
    } catch (error) {
        console.error('Connection failed:', error);
        appendChatMessage(`❌ Connection failed: ${error}`);
    }
};

window.sendMessage = function() {
    if (!connected || !gameClient) {
        appendChatMessage('❌ Not connected to game server');
        return;
    }
    
    const chatInput = document.getElementById('chatInput');
    const message = chatInput.value.trim();
    
    if (!message) {
        return;
    }
    
    try {
        gameClient.send_message(message);
        chatInput.value = '';
        
        // Show "SENDING..." that will disappear when server responds
        const timestamp = formatMilitaryTime(new Date());
        const myUsername = document.getElementById('userDisplay').textContent;
        const sendingDiv = addSendingMessage(`[${timestamp}] ${myUsername}: ${message}`, message);
        
        // Track it for removal when server responds
        pendingMessages.set(message.toLowerCase().trim(), sendingDiv);
        
    } catch (error) {
        console.error('Failed to send message:', error);
        appendChatMessage(`❌ Failed to send message: ${error}`);
    }
};

function addSendingMessage(messageText, rawMessage) {
    const chatMessages = document.getElementById('chatMessages');
    const messageDiv = document.createElement('div');
    messageDiv.textContent = messageText + " [SENDING...]";
    messageDiv.style.opacity = '0.6';
    messageDiv.style.fontStyle = 'italic';
    messageDiv.dataset.pending = 'true';
    messageDiv.dataset.rawMessage = rawMessage.toLowerCase().trim();
    
    chatMessages.appendChild(messageDiv);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    
    return messageDiv;
}

function appendChatMessage(message) {
    const chatMessages = document.getElementById('chatMessages');
    
    // Check if this is our own message - remove pending version
    // Updated regex to handle military time format HH:MM:SS.MS
    const serverMatch = message.match(/^\[(\d{2}:\d{2}:\d{2}\.\d{2})\]\s+(.+?):\s+(.+)$/);
    if (serverMatch) {
        const timestamp = serverMatch[1];
        const username = serverMatch[2];
        const messageContent = serverMatch[3].toLowerCase().trim();
        const myUsername = document.getElementById('userDisplay').textContent;
        
        if (username === myUsername && pendingMessages.has(messageContent)) {
            const pendingDiv = pendingMessages.get(messageContent);
            if (pendingDiv && pendingDiv.parentNode) {
                pendingDiv.remove();
            }
            pendingMessages.delete(messageContent);
        }
    }
    
    // Add server message
    const messageDiv = document.createElement('div');
    messageDiv.textContent = message;
    
    chatMessages.appendChild(messageDiv);
    
    // Auto-scroll to bottom
    chatMessages.scrollTop = chatMessages.scrollHeight;
    
    // Limit message history to last 100 messages
    while (chatMessages.children.length > 100) {
        chatMessages.removeChild(chatMessages.firstChild);
    }
}

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

function updateStats() {
    if (!connected || !gameClient) {
        return;
    }
    
    try {
        const position = gameClient.get_my_position();
        if (position) {
            const x = position[0];
            const y = position[1];
            document.getElementById('positionDisplay').textContent = `(${x}, ${y})`;
        }
    } catch (error) {
        console.error('Failed to get position:', error);
    }
}

function updateOnlinePlayersList() {
    const playersList = document.getElementById('playersList');
    playersList.innerHTML = '';
    
    if (onlinePlayers.size === 0) {
        const noPlayersDiv = document.createElement('div');
        noPlayersDiv.className = 'player-item';
        noPlayersDiv.innerHTML = `
            <span class="player-name">No players online</span>
            <span class="player-pos">--</span>
        `;
        playersList.appendChild(noPlayersDiv);
        return;
    }
    
    // Sort players by name
    const sortedPlayers = Array.from(onlinePlayers.values()).sort((a, b) => a.username.localeCompare(b.username));
    
    sortedPlayers.forEach(player => {
        const playerDiv = document.createElement('div');
        playerDiv.className = 'player-item';
        
        const isMe = player.username === document.getElementById('userDisplay').textContent;
        const nameStyle = isMe ? 'color: var(--iron-accent); font-weight: bold;' : '';
        
        playerDiv.innerHTML = `
            <span class="player-name" style="${nameStyle}">${player.username}${isMe ? ' (YOU)' : ''}</span>
            <span class="player-pos">(${player.x}, ${player.y})</span>
        `;
        
        playersList.appendChild(playerDiv);
    });
}

// Function to add/update a player in the online list
window.updatePlayerInList = function(username, x, y, health, resources) {
    onlinePlayers.set(username, { username, x, y, health, resources });
    updateOnlinePlayersList();
};

// Function to remove a player from the online list
window.removePlayerFromList = function(username) {
    onlinePlayers.delete(username);
    updateOnlinePlayersList();
};

function formatMilitaryTime(date) {
    const hours = String(date.getUTCHours()).padStart(2, '0');
    const minutes = String(date.getUTCMinutes()).padStart(2, '0');
    const seconds = String(date.getUTCSeconds()).padStart(2, '0');
    const milliseconds = String(Math.floor(date.getUTCMilliseconds() / 10)).padStart(2, '0');
    return `${hours}:${minutes}:${seconds}.${milliseconds}`;
}

function startPingSystem() {
    // Clear any existing ping interval
    if (pingInterval) {
        clearInterval(pingInterval);
    }
    
    // Start sending pings every 2 seconds
    pingInterval = setInterval(() => {
        if (connected && gameClient) {
            sendPing();
        }
    }, 2000);
    
    // Send initial ping
    setTimeout(() => {
        if (connected && gameClient) {
            sendPing();
        }
    }, 1000);
}

function sendPing() {
    if (!connected || !gameClient) return;
    
    pingStartTime = performance.now();
    try {
        // Send a special ping message that the server should echo back
        gameClient.send_message('__ping__');
    } catch (error) {
        console.error('Failed to send ping:', error);
    }
}

function updatePingDisplay(ping) {
    const pingDisplay = document.getElementById('pingDisplay');
    if (!pingDisplay) return;
    
    currentPing = ping;
    pingDisplay.textContent = `Ping: ${ping}ms`;
    
    // Update ping color based on latency
    pingDisplay.classList.remove('ping-good', 'ping-ok', 'ping-bad');
    if (ping < 50) {
        pingDisplay.classList.add('ping-good');
    } else if (ping < 150) {
        pingDisplay.classList.add('ping-ok');
    } else {
        pingDisplay.classList.add('ping-bad');
    }
}

// Function to be called from WASM when ping response is received
window.onPingReceived = function() {
    if (pingStartTime > 0) {
        const ping = Math.round(performance.now() - pingStartTime);
        updatePingDisplay(ping);
        pingStartTime = 0; // Reset for next ping
    }
}

// Function to be called from WASM when any message is received
window.onMessageReceived = function() {
    // This function is kept for compatibility but ping tracking moved to onPingReceived
}

// Initialize the application
run().catch(console.error); 