import init, { IronVeinClient } from './pkg/client.js';

let gameClient = null;
let connected = false;

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
    
    // Auto-update position display
    setInterval(updatePositionDisplay, 1000);
    
    // Listen for game state updates from WASM
    document.getElementById('gameCanvas').addEventListener('playerUpdate', (e) => {
        if (gameClient && e.detail) {
            try {
                const player = JSON.parse(e.detail);
                gameClient.update_player(player.username, player.x, player.y, player.health, player.resources);
                console.log('Updated player:', player.username, 'at', player.x, player.y);
            } catch (error) {
                console.error('Failed to update player:', error);
            }
        }
    });
    
    document.getElementById('gameCanvas').addEventListener('gameState', (e) => {
        if (gameClient && e.detail) {
            try {
                gameClient.update_all_players(e.detail);
                console.log('Updated all players from game state');
            } catch (error) {
                console.error('Failed to update game state:', error);
            }
        }
    });
}

window.connectToGame = async function() {
    const usernameInput = document.getElementById('usernameInput');
    const roomInput = document.getElementById('roomInput');
    const username = usernameInput.value.trim();
    const room = roomInput.value.trim();
    
    if (!username) {
        alert('Please enter a username!');
        usernameInput.focus();
        return;
    }
    
    if (!room) {
        alert('Please enter a room name!');
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
        
        // Add immediate feedback with different styling
        const chatMessages = document.getElementById('chatMessages');
        const messageDiv = document.createElement('div');
        messageDiv.style.opacity = '0.7';
        messageDiv.style.fontStyle = 'italic';
        messageDiv.setAttribute('data-pending', 'true');
        
        const timestamp = new Date().toLocaleTimeString();
        messageDiv.textContent = `[${timestamp}] [SENDING...] ${message}`;
        
        chatMessages.appendChild(messageDiv);
        chatMessages.scrollTop = chatMessages.scrollHeight;
        
    } catch (error) {
        console.error('Failed to send message:', error);
        appendChatMessage(`❌ Failed to send message: ${error}`);
    }
};

function appendChatMessage(message) {
    const chatMessages = document.getElementById('chatMessages');
    
    // Remove any pending messages first
    const pendingMessages = chatMessages.querySelectorAll('[data-pending="true"]');
    pendingMessages.forEach(msg => msg.remove());
    
    const messageDiv = document.createElement('div');
    
    // Add timestamp if not already included
    if (!message.includes('[') || !message.includes(']')) {
        const timestamp = new Date().toLocaleTimeString();
        messageDiv.textContent = `[${timestamp}] ${message}`;
    } else {
        messageDiv.textContent = message;
    }
    
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
    const statsPanel = document.getElementById('statsPanel');
    const chatInput = document.getElementById('chatInput');
    
    if (connected) {
        setupPanel.classList.add('hidden');
        statsPanel.classList.remove('hidden');
        chatInput.disabled = false;
        chatInput.placeholder = 'Type your message...';
    } else {
        setupPanel.classList.remove('hidden');
        statsPanel.classList.add('hidden');
        chatInput.disabled = true;
        chatInput.placeholder = 'Connect to chat...';
    }
}

function updateUserDisplay(username, room) {
    document.getElementById('userDisplay').textContent = username;
    document.getElementById('roomDisplay').textContent = `Room: ${room}`;
}

function updatePositionDisplay() {
    if (!connected || !gameClient) {
        return;
    }
    
    try {
        const position = gameClient.get_my_position();
        if (position) {
            const x = position[0];
            const y = position[1];
            document.getElementById('positionDisplay').textContent = `(${x}, ${y})`;
            
            // Update stats (placeholder for now)
            document.getElementById('healthStat').textContent = '100';
            document.getElementById('resourcesStat').textContent = '0';
        }
    } catch (error) {
        console.error('Failed to get position:', error);
    }
}

// Add some helper functions for the game
function updateGameStats(health, resources) {
    document.getElementById('healthStat').textContent = health;
    document.getElementById('resourcesStat').textContent = resources;
}

// Handle page visibility for performance
document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
        // Game is hidden, could pause updates
    } else {
        // Game is visible, resume updates
        if (connected && gameClient) {
            gameClient.render_game();
        }
    }
});

// Handle window resize
window.addEventListener('resize', () => {
    if (connected && gameClient) {
        // Re-render game on resize
        setTimeout(() => {
            gameClient.render_game();
        }, 100);
    }
});

// Initialize the application
run().catch(console.error);

// Export functions for debugging
window.gameClient = gameClient;
window.appendChatMessage = appendChatMessage;
window.updateGameStats = updateGameStats; 