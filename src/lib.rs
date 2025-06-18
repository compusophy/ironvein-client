use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, WebSocket, MessageEvent, Event, ErrorEvent, MouseEvent, HtmlCanvasElement, CanvasRenderingContext2d};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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

// Game constants
const GRID_SIZE: u32 = 64;
const CELL_SIZE: u32 = 10; // pixels per grid cell
const CANVAS_SIZE: u32 = GRID_SIZE * CELL_SIZE; // 640x640 pixels

// Game structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Player {
    username: String,
    x: u32,
    y: u32,
    room: String,
    health: u32,
    resources: u32,
}

// Game map functionality removed for simplicity - using server-side state only

// WebSocket message types (expanded for game)
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum WebSocketMessage {
    #[serde(rename = "join")]
    Join { username: String, room: String },
    #[serde(rename = "message")]
    Message { username: String, message: String, room: String },
    #[serde(rename = "chat_message")]
    ChatMessage(ChatMessage),
    #[serde(rename = "player_joined")]
    PlayerJoined { username: String, x: u32, y: u32 },
    #[serde(rename = "player_left")]
    PlayerLeft { username: String },
    #[serde(rename = "error")]
    Error { message: String },
    // Game-specific messages
    #[serde(rename = "move")]
    Move { username: String, x: u32, y: u32, room: String },
    #[serde(rename = "player_update")]
    PlayerUpdate { username: String, x: u32, y: u32, health: u32, resources: u32 },
    #[serde(rename = "game_state")]
    GameState { players: Vec<Player> },
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    id: String,
    username: String,
    message: String,
    timestamp: String,
    room: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserJoined {
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserLeft {
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorMessage {
    message: String,
}

#[wasm_bindgen]
pub struct IronVeinClient {
    username: String,
    room: String,
    websocket: Option<WebSocket>,
    // Game state
    players: HashMap<String, Player>,
    my_player: Option<Player>,
    canvas: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
}

#[wasm_bindgen]
impl IronVeinClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("🚀 IronVein Chat Client WASM module initialized!");
        Self {
            username: String::new(),
            room: String::new(),
            websocket: None,
            players: HashMap::new(),
            my_player: None,
            canvas: None,
            context: None,
        }
    }

    #[wasm_bindgen]
    pub fn setup_game_canvas(&mut self, canvas_id: &str) -> Result<(), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        canvas.set_width(CANVAS_SIZE);
        canvas.set_height(CANVAS_SIZE);

        let context = canvas
            .get_context("2d")?
            .ok_or("2d context not found")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Set up click handler for canvas
        let username_clone = self.username.clone();
        let room_clone = self.room.clone();
        let websocket_clone = self.websocket.clone();
        
        let click_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
            let canvas = event.target().unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
            let rect = canvas.get_bounding_client_rect();
            
            let x = ((event.client_x() as f64 - rect.x()) / CELL_SIZE as f64) as u32;
            let y = ((event.client_y() as f64 - rect.y()) / CELL_SIZE as f64) as u32;
            
            if x < GRID_SIZE && y < GRID_SIZE {
                console_log!("🎯 Click at grid position: ({}, {})", x, y);
                
                // Send move command to server
                if let Some(ref websocket) = websocket_clone {
                    let move_message = WebSocketMessage::Move {
                        username: username_clone.clone(),
                        x,
                        y,
                        room: room_clone.clone(),
                    };
                    
                    if let Ok(message_json) = serde_json::to_string(&move_message) {
                        let _ = websocket.send_with_str(&message_json);
                        console_log!("📤 Sent move command: ({}, {})", x, y);
                        
                        // Immediately update our own position optimistically
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        if let Some(pos_display) = document.get_element_by_id("positionDisplay") {
                            pos_display.set_text_content(Some(&format!("({}, {})", x, y)));
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MouseEvent)>);

        canvas.set_onclick(Some(click_callback.as_ref().unchecked_ref()));
        click_callback.forget();

        self.canvas = Some(canvas);
        self.context = Some(context);
        
        // Initial render
        self.render_game()?;
        
        console_log!("🎮 Game canvas setup complete! Click to move around the 64x64 grid.");
        Ok(())
    }

    #[wasm_bindgen]
    pub fn render_game(&self) -> Result<(), JsValue> {
        if let (Some(context), Some(_canvas)) = (&self.context, &self.canvas) {
            // Clear canvas
            context.clear_rect(0.0, 0.0, CANVAS_SIZE as f64, CANVAS_SIZE as f64);
            
            // Draw grid
            self.draw_grid(context)?;
            
            // Draw players
            self.draw_players(context)?;
            
            // Draw my player (highlighted)
            if let Some(ref my_player) = self.my_player {
                self.draw_my_player(context, my_player)?;
            }
        }
        Ok(())
    }

    fn draw_grid(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let _ = context.set_stroke_style(&"#333333".into());
        context.set_line_width(0.5);
        
        // Draw vertical lines
        for x in 0..=GRID_SIZE {
            let px = (x * CELL_SIZE) as f64;
            context.begin_path();
            context.move_to(px, 0.0);
            context.line_to(px, CANVAS_SIZE as f64);
            context.stroke();
        }
        
        // Draw horizontal lines
        for y in 0..=GRID_SIZE {
            let py = (y * CELL_SIZE) as f64;
            context.begin_path();
            context.move_to(0.0, py);
            context.line_to(CANVAS_SIZE as f64, py);
            context.stroke();
        }
        
        Ok(())
    }

    fn draw_players(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let _ = context.set_fill_style(&"#ff6b6b".into()); // Red for other players
        
        for player in self.players.values() {
            if Some(&player.username) != self.my_player.as_ref().map(|p| &p.username) {
                let px = (player.x * CELL_SIZE) as f64 + 2.0;
                let py = (player.y * CELL_SIZE) as f64 + 2.0;
                let size = (CELL_SIZE - 4) as f64;
                
                context.fill_rect(px, py, size, size);
                
                // Draw username
                let _ = context.set_fill_style(&"#000000".into());
                context.set_font("8px Arial");
                context.fill_text(&player.username, px, py - 2.0)?;
                let _ = context.set_fill_style(&"#ff6b6b".into());
            }
        }
        
        Ok(())
    }

    fn draw_my_player(&self, context: &CanvasRenderingContext2d, player: &Player) -> Result<(), JsValue> {
        let _ = context.set_fill_style(&"#4ecdc4".into()); // Teal for my player
        
        let px = (player.x * CELL_SIZE) as f64 + 2.0;
        let py = (player.y * CELL_SIZE) as f64 + 2.0;
        let size = (CELL_SIZE - 4) as f64;
        
        context.fill_rect(px, py, size, size);
        
        // Draw border to highlight
        let _ = context.set_stroke_style(&"#ffffff".into());
        context.set_line_width(2.0);
        context.stroke_rect(px, py, size, size);
        
        // Draw username
        let _ = context.set_fill_style(&"#000000".into());
        context.set_font("8px Arial");
        context.fill_text(&format!("{} (YOU)", player.username), px, py - 2.0)?;
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_user_info(&mut self, username: &str, room: &str) {
        self.username = username.to_string();
        self.room = room.to_string();
        
        // Initialize my player at random position
        let x = (js_sys::Math::random() * GRID_SIZE as f64) as u32;
        let y = (js_sys::Math::random() * GRID_SIZE as f64) as u32;
        
        self.my_player = Some(Player {
            username: username.to_string(),
            x,
            y,
            room: room.to_string(),
            health: 100,
            resources: 0,
        });
        
        console_log!("User info set: {} in room {} at position ({}, {})", username, room, x, y);
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
    pub fn connect(&mut self) -> Result<(), JsValue> {
        let server_url = Self::get_server_url();
        let ws_url = format!("{}/ws/{}", server_url, self.room);
        
        console_log!("Auto-detected server URL: {}", server_url);
        console_log!("Connecting to WebSocket: {}", ws_url);

        let websocket = WebSocket::new(&ws_url)?;
        
        // Set up WebSocket event handlers
        let username_clone = self.username.clone();
        let room_clone = self.room.clone();
        let websocket_clone = websocket.clone();
        let my_player_clone = self.my_player.clone();
        
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

            // Send initial position
            if let Some(ref player) = my_player_clone {
                let move_message = WebSocketMessage::Move {
                    username: player.username.clone(),
                    x: player.x,
                    y: player.y,
                    room: player.room.clone(),
                };
                
                if let Ok(message_json) = serde_json::to_string(&move_message) {
                    let _ = websocket_clone.send_with_str(&message_json);
                    console_log!("🎮 Sent initial position: ({}, {})", player.x, player.y);
                }
            }
        }) as Box<dyn FnMut(Event)>);
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // Handle incoming messages
        
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(message_text) = event.data().dyn_into::<js_sys::JsString>() {
                let message_str: String = message_text.into();
                
                // Try to parse as different message types
                if let Ok(ws_message) = serde_json::from_str::<WebSocketMessage>(&message_str) {
                    match ws_message {
                        WebSocketMessage::PlayerUpdate { username, x, y, health, resources } => {
                            console_log!("🎮 Player {} moved to ({}, {})", username, x, y);
                            
                            // Call update method on global gameClient
                            let window = web_sys::window().unwrap();
                            if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                                if !game_client.is_undefined() {
                                    if let Ok(update_fn) = js_sys::Reflect::get(&game_client, &"update_player".into()) {
                                        if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                                            let args = js_sys::Array::new();
                                            args.push(&username.clone().into());
                                            args.push(&(x as f64).into());
                                            args.push(&(y as f64).into());
                                            args.push(&(health as f64).into());
                                            args.push(&(resources as f64).into());
                                            let _ = func.apply(&game_client, &args);
                                            console_log!("✅ Updated player {} via global client", username);
                                        }
                                    }
                                }
                            }
                        }
                        WebSocketMessage::GameState { players } => {
                            console_log!("🌍 Received game state with {} players", players.len());
                            
                            // Call update_all_players on global gameClient
                            let window = web_sys::window().unwrap();
                            if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                                if !game_client.is_undefined() {
                                    if let Ok(players_json) = serde_json::to_string(&players) {
                                        if let Ok(update_fn) = js_sys::Reflect::get(&game_client, &"update_all_players".into()) {
                                            if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                                                let args = js_sys::Array::new();
                                                args.push(&players_json.into());
                                                let _ = func.apply(&game_client, &args);
                                                console_log!("✅ Updated all {} players via global client", players.len());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        WebSocketMessage::ChatMessage(chat_msg) => {
                            // Handle chat messages properly - THIS WAS BROKEN!
                            append_message(&format!("[{}] {}: {}", chat_msg.timestamp, chat_msg.username, chat_msg.message));
                        }
                        WebSocketMessage::PlayerJoined { username, x, y } => {
                            console_log!("🟢 Player {} joined at ({}, {})", username, x, y);
                            append_message(&format!("🟢 {} joined the battle at ({}, {})", username, x, y));
                            
                            // Add the new player to the map
                            let window = web_sys::window().unwrap();
                            if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                                if !game_client.is_undefined() {
                                    if let Ok(update_fn) = js_sys::Reflect::get(&game_client, &"update_player".into()) {
                                        if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                                            let args = js_sys::Array::new();
                                            args.push(&username.clone().into());
                                            args.push(&(x as f64).into());
                                            args.push(&(y as f64).into());
                                            args.push(&100f64.into());
                                            args.push(&0f64.into());
                                            let _ = func.apply(&game_client, &args);
                                            console_log!("✅ Added new player {} to map", username);
                                        }
                                    }
                                }
                            }
                        }
                        WebSocketMessage::PlayerLeft { username } => {
                            console_log!("🔴 Player {} left", username);
                            append_message(&format!("🔴 {} left the battle", username));
                        }
                        WebSocketMessage::Error { message } => {
                            console_log!("❌ Server error: {}", message);
                            append_message(&format!("❌ Error: {}", message));
                        }
                        _ => {
                            console_log!("📨 Other message: {}", message_str);
                        }
                    }
                } else {
                    console_log!("📨 Unknown message format: {}", message_str);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(|error_event: ErrorEvent| {
            console_log!("Connection failed: {:?}", error_event);
        }) as Box<dyn FnMut(ErrorEvent)>);
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onclose_callback = Closure::wrap(Box::new(|_event: Event| {
            console_log!("WebSocket connection closed");
            append_message("❌ Connection lost. Refresh to reconnect.");
        }) as Box<dyn FnMut(Event)>);
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        self.websocket = Some(websocket);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn join_room(&self) -> Result<(), JsValue> {
        // Join room is now handled automatically in the onopen callback
        console_log!("Join room will happen automatically when WebSocket connects");
        Ok(())
    }

    #[wasm_bindgen]
    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ref websocket) = self.websocket {
            let chat_message = WebSocketMessage::Message {
                username: self.username.clone(),
                message: message.to_string(),
                room: self.room.clone(),
            };
            
            let message_json = serde_json::to_string(&chat_message)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            
            websocket.send_with_str(&message_json)?;
            console_log!("Sent message: {}", message);
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_grid_size(&self) -> u32 {
        GRID_SIZE
    }

    #[wasm_bindgen]
    pub fn get_my_position(&self) -> Option<js_sys::Array> {
        if let Some(ref player) = self.my_player {
            let position = js_sys::Array::new();
            position.push(&JsValue::from(player.x));
            position.push(&JsValue::from(player.y));
            Some(position)
        } else {
            None
        }
    }

    // New methods to handle game state updates
    #[wasm_bindgen]
    pub fn update_player(&mut self, username: &str, x: u32, y: u32, health: u32, resources: u32) {
        let player = Player {
            username: username.to_string(),
            x, y, health, resources,
            room: self.room.clone(),
        };
        
        // Update my player if it's me
        if username == self.username {
            self.my_player = Some(player.clone());
        }
        
        // Update players HashMap
        self.players.insert(username.to_string(), player);
        
        // Re-render the game
        let _ = self.render_game();
    }

    #[wasm_bindgen] 
    pub fn update_all_players(&mut self, players_json: &str) -> Result<(), JsValue> {
        if let Ok(players) = serde_json::from_str::<Vec<Player>>(players_json) {
            // Clear current players
            self.players.clear();
            
            // Add all players
            for player in players {
                if player.username == self.username {
                    self.my_player = Some(player.clone());
                }
                self.players.insert(player.username.clone(), player);
            }
            
            // Re-render the game
            self.render_game()?;
        }
        Ok(())
    }
}

// Helper functions
fn append_message(message: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    if let Some(chat_messages) = document.get_element_by_id("chatMessages") {
        let new_message = document.create_element("div").unwrap();
        new_message.set_text_content(Some(message));
        chat_messages.append_child(&new_message).unwrap();
        
        // Auto-scroll to bottom
        chat_messages.set_scroll_top(chat_messages.scroll_height());
    }
}

// Macro for console logging
macro_rules! console_log {
    ($($t:tt)*) => (console::log_1(&format!($($t)*).into()))
}

use console_log;

// Timestamp formatting removed - using raw timestamps from server

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🚀 IronVein Chat Client WASM module initialized!");
} 