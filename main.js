import init, { ChatClient } from './pkg/client.js';

let chatClient = null;

async function initWasm() {
    try {
        await init();
        chatClient = new ChatClient();
        updateConnectionStatus('🔄 Ready to connect', 'disconnected');
        console.log('🚀 IronVein Chat WASM module loaded successfully!');
    } catch (error) {
        console.error('Failed to load WASM:', error);
        updateConnectionStatus('❌ Failed to load WASM', 'disconnected');
    }
}

function updateConnectionStatus(message, status) {
    const statusElement = document.getElementById('connectionStatus');
    statusElement.textContent = message;
    statusElement.className = `connection-status ${status}`;
}

function addSystemMessage(message) {
    const messagesContainer = document.getElementById('messages');
    const messageDiv = document.createElement('div');
    messageDiv.className = 'system-message';
    messageDiv.textContent = message;
    messagesContainer.appendChild(messageDiv);
    scrollToBottom();
}

function scrollToBottom() {
    const messagesContainer = document.querySelector('.messages-container');
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
}

// Event handlers
document.addEventListener('DOMContentLoaded', () => {
    initWasm();

    // Connect button handler
    const connectBtn = document.getElementById('connectBtn');
    const joinForm = document.getElementById('joinForm');
    const messageInputContainer = document.getElementById('messageInputContainer');
    const usernameInput = document.getElementById('username');
    const roomInput = document.getElementById('room');

    connectBtn.addEventListener('click', async () => {
        const username = usernameInput.value.trim();
        const room = roomInput.value.trim();

        if (!username || !room) {
            addSystemMessage('❌ Please enter both username and room name');
            return;
        }

        if (!chatClient) {
            addSystemMessage('❌ Chat client not initialized');
            return;
        }

        try {
            // Set user info
            chatClient.set_user_info(username, room);
            
            // Update UI
            connectBtn.disabled = true;
            connectBtn.textContent = 'Connecting...';
            updateConnectionStatus('🔄 Connecting...', 'disconnected');

            // Connect to WebSocket
            await chatClient.connect();
            
            // Join the room
            await chatClient.join_room();

            // Update UI for connected state
            joinForm.classList.add('hidden');
            messageInputContainer.classList.remove('hidden');
            updateConnectionStatus(`✅ Connected as ${username} in #${room}`, 'connected');
            
            addSystemMessage(`🎉 Connected to room #${room} as ${username}`);
            addSystemMessage(`🔗 ${chatClient.get_server_info()}`);

        } catch (error) {
            console.error('Connection failed:', error);
            addSystemMessage(`❌ Connection failed: ${error}`);
            connectBtn.disabled = false;
            connectBtn.textContent = 'Join Chat';
            updateConnectionStatus('❌ Connection failed', 'disconnected');
        }
    });

    // Message form handler
    const messageForm = document.getElementById('messageForm');
    const messageInput = document.getElementById('messageInput');

    messageForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        
        const message = messageInput.value.trim();
        if (!message || !chatClient) return;

        try {
            await chatClient.send_message(message);
            messageInput.value = '';
        } catch (error) {
            console.error('Failed to send message:', error);
            addSystemMessage(`❌ Failed to send message: ${error}`);
        }
    });

    // Enter key to connect when in username/room inputs
    [usernameInput, roomInput].forEach(input => {
        input.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                connectBtn.click();
            }
        });
    });

    // Auto-focus username input
    usernameInput.focus();
});

// Handle page unload - disconnect gracefully
window.addEventListener('beforeunload', () => {
    if (chatClient && chatClient.is_connected()) {
        chatClient.disconnect();
    }
}); 