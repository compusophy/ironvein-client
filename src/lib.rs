use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, Event};
use serde::{Deserialize, Serialize};
use js_sys::Date;

// Import the `console.log` function from the Web API
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Chat message structure matching the server
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    id: String,
    username: String,
    message: String,
    timestamp: String,
    room: String,
}

// WebSocket message types matching the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WebSocketMessage {
    #[serde(rename = "join")]
    Join { username: String, room: String },
    #[serde(rename = "message")]
    Message { message: String },
    #[serde(rename = "chat_message")]
    ChatMessage(ChatMessage),
    #[serde(rename = "user_joined")]
    UserJoined { username: String, room: String },
    #[serde(rename = "user_left")]
    UserLeft { username: String, room: String },
    #[serde(rename = "error")]
    Error { message: String },
}

#[wasm_bindgen]
pub struct ChatClient {
    websocket: Option<WebSocket>,
    connected: bool,
    username: String,
    room: String,
    server_url: String,
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ChatClient {
        // Auto-detect server URL based on current location
        let server_url = Self::get_server_url();
        console_log!("Auto-detected server URL: {}", server_url);
        
        ChatClient {
            websocket: None,
            connected: false,
            username: String::new(),
            room: "general".to_string(),
            server_url,
        }
    }

    fn get_server_url() -> String {
        let window = web_sys::window().unwrap();
        let location = window.location();
        
        // Try to get the protocol and host from current page
        if let (Ok(protocol), Ok(host)) = (location.protocol(), location.host()) {
            // Convert http/https to ws/wss
            let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
            
            // Check if we're running locally
            if host.contains("localhost") || host.contains("127.0.0.1") {
                // In development, assume server is on localhost:8080
                format!("{}//localhost:8080", ws_protocol)
            } else {
                // In production on Railway, replace client domain with server domain
                // ironvein-client-production.up.railway.app -> ironvein-server-production.up.railway.app
                if host.contains("ironvein-client") {
                    let server_host = host.replace("ironvein-client", "ironvein-server");
                    format!("{}//{}", ws_protocol, server_host)
                } else {
                    // Generic Railway pattern
                    format!("{}//{}", ws_protocol, "ironvein-server-production.up.railway.app")
                }
            }
        } else {
            // Fallback
            "ws://localhost:8080".to_string()
        }
    }

    #[wasm_bindgen]
    pub fn set_user_info(&mut self, username: String, room: String) {
        self.username = username;
        self.room = room;
        console_log!("User info set: {} in room {}", self.username, self.room);
    }

    #[wasm_bindgen]
    pub fn connect(&mut self) -> Result<(), JsValue> {
        if self.connected {
            return Ok(());
        }

        let ws_url = format!("{}/ws/{}", self.server_url, self.room);
        console_log!("Connecting to WebSocket: {}", ws_url);

        let websocket = WebSocket::new(&ws_url)?;
        
        // Set up WebSocket event handlers
        let username_clone = self.username.clone();
        let room_clone = self.room.clone();
        let websocket_clone = websocket.clone();
        
        let onopen_callback = Closure::wrap(Box::new(move |_event: Event| {
            console_log!("WebSocket connected!");
            
            // Auto-join room when connection opens
            let join_message = WebSocketMessage::Join {
                username: username_clone.clone(),
                room: room_clone.clone(),
            };
            
            if let Ok(message_json) = serde_json::to_string(&join_message) {
                let _ = websocket_clone.send_with_str(&message_json);
                console_log!("Auto-joined room {} as {}", room_clone, username_clone);
            }
        }) as Box<dyn FnMut(Event)>);
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
                let message_text = text.as_string().unwrap_or_default();
                console_log!("Received message: {}", message_text);
                
                // Parse and handle the message
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&message_text) {
                    Self::handle_message(ws_msg);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
            console_log!("WebSocket error: {:?}", event);
        }) as Box<dyn FnMut(ErrorEvent)>);
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onclose_callback = Closure::wrap(Box::new(move |_event: Event| {
            console_log!("WebSocket disconnected");
        }) as Box<dyn FnMut(Event)>);
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        self.websocket = Some(websocket);
        self.connected = true;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn join_room(&self) -> Result<(), JsValue> {
        // Join room is now handled automatically in the onopen callback
        console_log!("Join room will happen automatically when WebSocket connects");
        Ok(())
    }

    #[wasm_bindgen]
    pub fn send_message(&self, message: String) -> Result<(), JsValue> {
        if let Some(ref websocket) = self.websocket {
            let chat_message = WebSocketMessage::Message { message };
            
            let message_json = serde_json::to_string(&chat_message)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            
            websocket.send_with_str(&message_json)?;
            console_log!("Sent message: {}", message_json);
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn disconnect(&mut self) {
        if let Some(websocket) = self.websocket.take() {
            let _ = websocket.close();
            self.connected = false;
            console_log!("Disconnected from WebSocket");
        }
    }

    #[wasm_bindgen]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    #[wasm_bindgen]
    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    #[wasm_bindgen]
    pub fn get_room(&self) -> String {
        self.room.clone()
    }

    #[wasm_bindgen]
    pub fn get_server_info(&self) -> String {
        format!("Connected to: {}", self.server_url)
    }

    // Static method to handle incoming messages
    fn handle_message(message: WebSocketMessage) {
        match message {
            WebSocketMessage::ChatMessage(chat_msg) => {
                Self::display_chat_message(chat_msg);
            }
            WebSocketMessage::UserJoined { username, room } => {
                Self::display_system_message(&format!("{} joined {}", username, room));
            }
            WebSocketMessage::UserLeft { username, room } => {
                Self::display_system_message(&format!("{} left {}", username, room));
            }
            WebSocketMessage::Error { message } => {
                Self::display_error_message(&message);
            }
            _ => {}
        }
    }

    fn display_chat_message(chat_msg: ChatMessage) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(messages_container) = document.get_element_by_id("messages") {
            let message_div = document.create_element("div").unwrap();
            message_div.set_class_name("message");
            
            let timestamp = Self::format_timestamp(&chat_msg.timestamp);
            let content = format!(
                "<span class=\"timestamp\">[{}]</span> <span class=\"username\">{}</span>: <span class=\"text\">{}</span>",
                timestamp, chat_msg.username, chat_msg.message
            );
            
            message_div.set_inner_html(&content);
            messages_container.append_child(&message_div).unwrap();
            
            // Scroll to bottom
            messages_container.set_scroll_top(messages_container.scroll_height());
        }
    }

    fn display_system_message(message: &str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(messages_container) = document.get_element_by_id("messages") {
            let message_div = document.create_element("div").unwrap();
            message_div.set_class_name("system-message");
            message_div.set_inner_html(&format!("🔔 {}", message));
            
            messages_container.append_child(&message_div).unwrap();
            messages_container.set_scroll_top(messages_container.scroll_height());
        }
    }

    fn display_error_message(message: &str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(messages_container) = document.get_element_by_id("messages") {
            let message_div = document.create_element("div").unwrap();
            message_div.set_class_name("error-message");
            message_div.set_inner_html(&format!("❌ Error: {}", message));
            
            messages_container.append_child(&message_div).unwrap();
            messages_container.set_scroll_top(messages_container.scroll_height());
        }
    }

    fn format_timestamp(timestamp: &str) -> String {
        // Simple timestamp formatting - just use current time
        let date = Date::new_0();
        let time = date.to_time_string().as_string().unwrap_or_default();
        time
    }
}

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🚀 IronVein Chat Client WASM module initialized!");
} 