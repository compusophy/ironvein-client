use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Timelike;
use std::cell::RefCell;
use std::rc::Rc;

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
const CELL_SIZE: u32 = 16;
const CANVAS_SIZE: u32 = GRID_SIZE * CELL_SIZE;

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

// WebSocket message types
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
    #[serde(rename = "move")]
    Move { username: String, x: u32, y: u32, room: String },
    #[serde(rename = "player_update")]
    PlayerUpdate { username: String, x: u32, y: u32, health: u32, resources: u32 },
    #[serde(rename = "game_state")]
    GameState { players: Vec<Player> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    id: serde_json::Value,
    username: String,
    message: String,
    timestamp: serde_json::Value,
    room: String,
}

#[wasm_bindgen]
pub struct IronVeinClient {
    username: String,
    room: String,
    websocket: Option<WebSocket>,
    players: HashMap<String, Player>,
    my_player: Option<Player>,
    canvas: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    game_loop_id: Option<i32>,
    pending_messages: Rc<RefCell<HashMap<String, web_sys::Element>>>,
}

#[wasm_bindgen]
impl IronVeinClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("🚀 IronVein Game Engine (Rust) initialized!");
        Self {
            username: String::new(),
            room: String::new(),
            websocket: None,
            players: HashMap::new(),
            my_player: None,
            canvas: None,
            context: None,
            game_loop_id: None,
            pending_messages: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    #[wasm_bindgen]
    pub fn set_user_info(&mut self, username: &str, room: &str) {
        self.username = username.to_string();
        self.room = room.to_string();
        console_log!("User info set: {} in room {}", username, room);
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

        self.canvas = Some(canvas);
        self.context = Some(context);
        
        console_log!("🎮 Game canvas setup complete!");
        Ok(())
    }

    #[wasm_bindgen]
    pub fn connect(&mut self) -> Result<(), JsValue> {
        let server_url = Self::get_server_url();
        let ws_url = format!("{}/ws/{}", server_url, self.room);
        
        console_log!("Connecting to WebSocket: {}", ws_url);
        let websocket = WebSocket::new(&ws_url)?;
        
        // Set up WebSocket event handlers
        let username = self.username.clone();
        let room = self.room.clone();
        
        // Store websocket reference for move commands
        self.websocket = Some(websocket.clone());
        
        // Setup all WebSocket handlers
        self.setup_websocket_handlers(&websocket, username, room)?;
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn connect_to_server(&mut self) -> Result<(), JsValue> {
        let server_url = Self::get_server_url();
        let ws_url = format!("{}/ws/{}", server_url, self.room);
        
        console_log!("Connecting to WebSocket: {}", ws_url);
        let websocket = WebSocket::new(&ws_url)?;
        
        // Set up WebSocket event handlers but don't auto-join
        let username = self.username.clone();
        let room = self.room.clone();
        
        // Store websocket reference
        self.websocket = Some(websocket.clone());
        
        // Setup WebSocket handlers without auto-join
        self.setup_websocket_handlers_lobby_only(&websocket, username, room)?;
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn join_battle(&self) -> Result<(), JsValue> {
        if !self.is_websocket_connected() {
            return Err(JsValue::from_str("Not connected to server"));
        }
        
        if let Some(ref websocket) = self.websocket {
            // Send join message to spawn player
            let join_message = WebSocketMessage::Join {
                username: self.username.clone(),
                room: self.room.clone(),
            };
            
            if let Ok(message_json) = serde_json::to_string(&join_message) {
                websocket.send_with_str(&message_json)?;
                console_log!("🏠 Joined battle as {} in room {}", self.username, self.room);
                
                // Setup click handler and start game loop
                let window = web_sys::window().unwrap();
                if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                    if let Ok(setup_fn) = js_sys::Reflect::get(&game_client, &"setup_click_handler".into()) {
                        if let Ok(func) = setup_fn.dyn_into::<js_sys::Function>() {
                            let _ = func.call0(&game_client);
                        }
                    }
                    if let Ok(start_fn) = js_sys::Reflect::get(&game_client, &"start_game_loop".into()) {
                        if let Ok(func) = start_fn.dyn_into::<js_sys::Function>() {
                            let _ = func.call0(&game_client);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn send_ping(&self) -> Result<(), JsValue> {
        if !self.is_websocket_connected() {
            return Ok(()); // Silent fail for pings
        }
        
        if let Some(ref websocket) = self.websocket {
            // Use minimal ping payload for efficiency
            let ping_message = WebSocketMessage::Message {
                username: self.username.clone(),
                message: "p".to_string(), // Minimal payload
                room: self.room.clone(),
            };
            
            if let Ok(message_json) = serde_json::to_string(&ping_message) {
                let _ = websocket.send_with_str(&message_json);
                // Silent operation - no logging for pings
            }
        }
        
        Ok(())
    }

    fn setup_websocket_handlers(&self, websocket: &WebSocket, username: String, room: String) -> Result<(), JsValue> {
        // Store reference to self for callbacks
        let websocket_for_join = websocket.clone();
        
        // OnOpen - join room and setup game
        let username_clone = username.clone();
        let room_clone = room.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_event: Event| {
            console_log!("🌐 WebSocket connected!");
            
            // Auto-join room
            let join_message = WebSocketMessage::Join {
                username: username_clone.clone(),
                room: room_clone.clone(),
            };
            
            if let Ok(message_json) = serde_json::to_string(&join_message) {
                let _ = websocket_for_join.send_with_str(&message_json);
                console_log!("🏠 Auto-joined room {} as {}", room_clone, username_clone);
            }
            
            // Setup click handler and start game loop
            let window = web_sys::window().unwrap();
            if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                if let Ok(setup_fn) = js_sys::Reflect::get(&game_client, &"setup_click_handler".into()) {
                    if let Ok(func) = setup_fn.dyn_into::<js_sys::Function>() {
                        let _ = func.call0(&game_client);
                    }
                }
                if let Ok(start_fn) = js_sys::Reflect::get(&game_client, &"start_game_loop".into()) {
                    if let Ok(func) = start_fn.dyn_into::<js_sys::Function>() {
                        let _ = func.call0(&game_client);
                    }
                }
            }
        }) as Box<dyn FnMut(Event)>);
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // OnMessage - handle all server messages
        let username_for_msg = username.clone();
        let pending_messages = self.pending_messages.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(message_str) = event.data().dyn_into::<js_sys::JsString>() {
                let message_str = String::from(message_str);
                
                match serde_json::from_str::<WebSocketMessage>(&message_str) {
                    Ok(parsed_message) => {
                        match parsed_message {
                            WebSocketMessage::PlayerJoined { username, x, y } => {
                                console_log!("🟢 Player {} joined at ({}, {})", username, x, y);
                                Self::update_player_list(&username, x, y, 100, 0);
                                Self::update_game_client_player(&username, x, y, 100, 0);
                            }
                            WebSocketMessage::PlayerUpdate { username, x, y, health, resources } => {
                                console_log!("🎮 Player {} moved to ({}, {})", username, x, y);
                                Self::update_player_list(&username, x, y, health, resources);
                                Self::update_game_client_player(&username, x, y, health, resources);
                                Self::update_position_display(&username, x, y);
                            }
                            WebSocketMessage::PlayerLeft { username } => {
                                console_log!("🔴 Player {} left", username);
                                Self::remove_player_from_list(&username);
                            }
                            WebSocketMessage::GameState { players } => {
                                console_log!("🌍 Received game state with {} players", players.len());
                                for player in &players {
                                    Self::update_player_list(&player.username, player.x, player.y, player.health, player.resources);
                                }
                                Self::update_all_game_players(&players);
                            }
                            WebSocketMessage::ChatMessage(chat_msg) => {
                                // Handle ping responses with backward compatibility
                                if (chat_msg.message == "__ping__" || chat_msg.message == "p") && chat_msg.username == username_for_msg {
                                    Self::handle_ping_response();
                                    return;
                                }
                                Self::handle_chat_message(chat_msg, &pending_messages);
                            }
                            WebSocketMessage::Error { message } => {
                                console_log!("❌ Server error: {}", message);
                                Self::append_chat_message(&format!("❌ Error: {}", message));
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        console_log!("❌ Failed to parse message: {}", e);
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // OnError - handle connection errors
        let onerror_callback = Closure::wrap(Box::new(|error_event: ErrorEvent| {
            console_log!("❌ WebSocket connection error: {:?}", error_event);
            Self::append_chat_message("❌ Connection error - please refresh to reconnect");
        }) as Box<dyn FnMut(ErrorEvent)>);
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // OnClose - handle connection close
        let onclose_callback = Closure::wrap(Box::new(|close_event: CloseEvent| {
            console_log!("🔌 WebSocket connection closed. Code: {}, Reason: {}", 
                close_event.code(), close_event.reason());
            Self::append_chat_message("🔌 Connection lost. Please refresh to reconnect.");
        }) as Box<dyn FnMut(CloseEvent)>);
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        Ok(())
    }

    fn setup_websocket_handlers_lobby_only(&self, websocket: &WebSocket, username: String, room: String) -> Result<(), JsValue> {
        // OnOpen - connect but don't auto-join battle
        let onopen_callback = Closure::wrap(Box::new(move |_event: Event| {
            console_log!("🌐 WebSocket connected to lobby!");
            // Don't auto-join - user will manually join battle later
        }) as Box<dyn FnMut(Event)>);
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // OnMessage - handle lobby messages (chat only, no game events yet)
        let username_for_msg = username.clone();
        let pending_messages = self.pending_messages.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(message_str) = event.data().dyn_into::<js_sys::JsString>() {
                let message_str = String::from(message_str);
                
                match serde_json::from_str::<WebSocketMessage>(&message_str) {
                    Ok(parsed_message) => {
                        match parsed_message {
                            WebSocketMessage::ChatMessage(chat_msg) => {
                                // Handle ping responses with backward compatibility
                                if (chat_msg.message == "__ping__" || chat_msg.message == "p") && chat_msg.username == username_for_msg {
                                    Self::handle_ping_response();
                                    return;
                                }
                                Self::handle_chat_message(chat_msg, &pending_messages);
                            }
                            WebSocketMessage::Error { message } => {
                                console_log!("❌ Server error: {}", message);
                                Self::append_chat_message(&format!("❌ Error: {}", message));
                            }
                            _ => {
                                // Ignore game events in lobby mode
                            }
                        }
                    }
                    Err(e) => {
                        console_log!("❌ Failed to parse message: {}", e);
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // OnError - handle connection errors
        let onerror_callback = Closure::wrap(Box::new(|error_event: ErrorEvent| {
            console_log!("❌ WebSocket connection error: {:?}", error_event);
            Self::append_chat_message("❌ Connection error - please refresh to reconnect");
        }) as Box<dyn FnMut(ErrorEvent)>);
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // OnClose - handle connection close
        let onclose_callback = Closure::wrap(Box::new(|close_event: CloseEvent| {
            console_log!("🔌 WebSocket connection closed. Code: {}, Reason: {}", 
                close_event.code(), close_event.reason());
            Self::append_chat_message("🔌 Connection lost. Please refresh to reconnect.");
        }) as Box<dyn FnMut(CloseEvent)>);
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        Ok(())
    }

    #[wasm_bindgen]
    pub fn setup_click_handler(&self) -> Result<(), JsValue> {
        if let Some(ref canvas) = self.canvas {
            let click_callback = Closure::wrap(Box::new(move |event: MouseEvent| {
                let canvas = event.target().unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
                let rect = canvas.get_bounding_client_rect();
                
                let canvas_width = rect.width();
                let canvas_height = rect.height();
                
                let x = ((event.client_x() as f64 - rect.x()) / canvas_width * GRID_SIZE as f64) as u32;
                let y = ((event.client_y() as f64 - rect.y()) / canvas_height * GRID_SIZE as f64) as u32;
                
                if x < GRID_SIZE && y < GRID_SIZE {
                    console_log!("🎯 Click at grid position: ({}, {})", x, y);
                    
                    // Send move command directly
                    let window = web_sys::window().unwrap();
                    if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                        if let Ok(send_move_fn) = js_sys::Reflect::get(&game_client, &"send_move_command".into()) {
                            if let Ok(func) = send_move_fn.dyn_into::<js_sys::Function>() {
                                let args = js_sys::Array::new();
                                args.push(&(x as f64).into());
                                args.push(&(y as f64).into());
                                let _ = func.apply(&game_client, &args);
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(MouseEvent)>);

            canvas.set_onclick(Some(click_callback.as_ref().unchecked_ref()));
            click_callback.forget();
            
            console_log!("🖱️ Click handler setup complete! Click to move around the grid.");
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn send_move_command(&self, x: u32, y: u32) -> Result<(), JsValue> {
        if !self.is_websocket_connected() {
            console_log!("❌ WebSocket not connected, cannot send move command");
            return Ok(());
        }
        
        if let Some(ref websocket) = self.websocket {
            let move_message = WebSocketMessage::Move {
                username: self.username.clone(),
                x,
                y,
                room: self.room.clone(),
            };
            
            if let Ok(message_json) = serde_json::to_string(&move_message) {
                match websocket.send_with_str(&message_json) {
                    Ok(_) => {
                        console_log!("📤 Sent move command: ({}, {})", x, y);
                        // Optimistic update
                        Self::update_position_display(&self.username, x, y);
                    }
                    Err(e) => {
                        console_log!("❌ Failed to send move command: {:?}", e);
                    }
                }
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if !self.is_websocket_connected() {
            console_log!("❌ WebSocket not connected, cannot send message");
            // Only show error for non-ping messages
            if message != "__ping__" && message != "p" {
                Self::append_chat_message("❌ Not connected to server");
            }
            return Ok(());
        }
        
        if let Some(ref websocket) = self.websocket {
            let chat_message = WebSocketMessage::Message {
                username: self.username.clone(),
                message: message.to_string(),
                room: self.room.clone(),
            };
            
            let message_json = serde_json::to_string(&chat_message)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            
            match websocket.send_with_str(&message_json) {
                Ok(_) => {
                    // Only log and add to pending for non-ping messages
                    if message == "__ping__" || message == "p" {
                        // Silent ping - don't log or add to chat
                    } else {
                        console_log!("💬 Sent chat message: {}", message);
                        // Add to pending messages only for real chat messages
                        Self::add_pending_message(message, &self.pending_messages);
                    }
                }
                Err(e) => {
                    console_log!("❌ Failed to send message: {:?}", e);
                    // Only show error for non-ping messages
                    if message != "__ping__" && message != "p" {
                        Self::append_chat_message("❌ Failed to send message - connection lost");
                    }
                }
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn start_game_loop(&mut self) -> Result<(), JsValue> {
        let game_loop = Closure::wrap(Box::new(move |_timestamp: f64| {
            // Call render on global gameClient at 60fps
            let window = web_sys::window().unwrap();
            if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
                if let Ok(render_fn) = js_sys::Reflect::get(&game_client, &"render_game".into()) {
                    if let Ok(func) = render_fn.dyn_into::<js_sys::Function>() {
                        let _ = func.call0(&game_client);
                    }
                }
            }
            
            // Schedule next frame
            if let Ok(raf) = js_sys::Reflect::get(&window, &"requestAnimationFrame".into()) {
                if let Ok(func) = raf.dyn_into::<js_sys::Function>() {
                    let callback = js_sys::Reflect::get(&window, &"gameLoopCallback".into()).unwrap();
                    let _ = func.call1(&window, &callback);
                }
            }
        }) as Box<dyn FnMut(f64)>);
        
        // Store callback globally for requestAnimationFrame
        let window = web_sys::window().unwrap();
        js_sys::Reflect::set(&window, &"gameLoopCallback".into(), game_loop.as_ref().unchecked_ref())?;
        game_loop.forget();
        
        // Start the loop
        if let Ok(raf) = js_sys::Reflect::get(&window, &"requestAnimationFrame".into()) {
            if let Ok(func) = raf.dyn_into::<js_sys::Function>() {
                let callback = js_sys::Reflect::get(&window, &"gameLoopCallback".into()).unwrap();
                let _id = func.call1(&window, &callback)?;
                // Store the ID if needed for cancellation
            }
        }
        
        console_log!("🎮 60fps game loop started!");
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
        }
        Ok(())
    }

    fn draw_grid(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.set_stroke_style_str("#333");
        context.set_line_width(0.5);
        
        for i in 0..=GRID_SIZE {
            let pos = (i * CELL_SIZE) as f64;
            context.begin_path();
            context.move_to(pos, 0.0);
            context.line_to(pos, CANVAS_SIZE as f64);
            context.stroke();
            
            context.begin_path();
            context.move_to(0.0, pos);
            context.line_to(CANVAS_SIZE as f64, pos);
            context.stroke();
        }
        Ok(())
    }

    fn draw_players(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        for player in self.players.values() {
            let x = (player.x * CELL_SIZE) as f64;
            let y = (player.y * CELL_SIZE) as f64;
            
            if player.username == self.username {
                // Draw self in green
                context.set_fill_style_str("#4CAF50");
            } else {
                // Draw others in red
                context.set_fill_style_str("#F44336");
            }
            
            context.fill_rect(x + 2.0, y + 2.0, (CELL_SIZE - 4) as f64, (CELL_SIZE - 4) as f64);
            
            // Draw username
            context.set_fill_style_str("white");
            context.set_font("10px Arial");
            context.fill_text(&player.username, x + 2.0, y + CELL_SIZE as f64 - 2.0)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn update_player(&mut self, username: &str, x: u32, y: u32, health: u32, resources: u32) {
        let player = Player {
            username: username.to_string(),
            x, y, health, resources,
            room: self.room.clone(),
        };
        
        if username == self.username {
            self.my_player = Some(player.clone());
        }
        
        self.players.insert(username.to_string(), player);
    }

    #[wasm_bindgen]
    pub fn update_all_players(&mut self, players_json: &str) -> Result<(), JsValue> {
        if let Ok(players) = serde_json::from_str::<Vec<Player>>(players_json) {
            self.players.clear();
            
            for player in players {
                if player.username == self.username {
                    self.my_player = Some(player.clone());
                }
                self.players.insert(player.username.clone(), player);
            }
        }
        Ok(())
    }

    // Static helper functions for UI updates
    fn get_server_url() -> String {
        let window = web_sys::window().unwrap();
        let location = window.location();
        
        if let Ok(hostname) = location.hostname() {
            console_log!("🌐 Detected hostname: {}", hostname);
            if hostname.contains("localhost") || hostname.contains("127.0.0.1") {
                // Check if local server is available, otherwise use production
                console_log!("🏠 Local development detected, but using production server for reliability");
                "wss://ironvein-server-production.up.railway.app".to_string()
            } else {
                console_log!("🌍 Production environment detected");
                "wss://ironvein-server-production.up.railway.app".to_string()
            }
        } else {
            console_log!("🌍 Fallback to production server");
            "wss://ironvein-server-production.up.railway.app".to_string()
        }
    }

    fn update_player_list(username: &str, x: u32, y: u32, health: u32, resources: u32) {
        let window = web_sys::window().unwrap();
        if let Ok(update_fn) = js_sys::Reflect::get(&window, &"updatePlayerInList".into()) {
            if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                let args = js_sys::Array::new();
                args.push(&username.into());
                args.push(&(x as f64).into());
                args.push(&(y as f64).into());
                args.push(&(health as f64).into());
                args.push(&(resources as f64).into());
                let _ = func.apply(&window, &args);
            }
        }
    }

    fn remove_player_from_list(username: &str) {
        let window = web_sys::window().unwrap();
        if let Ok(remove_fn) = js_sys::Reflect::get(&window, &"removePlayerFromList".into()) {
            if let Ok(func) = remove_fn.dyn_into::<js_sys::Function>() {
                let args = js_sys::Array::new();
                args.push(&username.into());
                let _ = func.apply(&window, &args);
            }
        }
    }

    fn update_game_client_player(username: &str, x: u32, y: u32, health: u32, resources: u32) {
        let window = web_sys::window().unwrap();
        if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
            if let Ok(update_fn) = js_sys::Reflect::get(&game_client, &"update_player".into()) {
                if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                    let args = js_sys::Array::new();
                    args.push(&username.into());
                    args.push(&(x as f64).into());
                    args.push(&(y as f64).into());
                    args.push(&(health as f64).into());
                    args.push(&(resources as f64).into());
                    let _ = func.apply(&game_client, &args);
                }
            }
        }
    }

    fn update_all_game_players(players: &[Player]) {
        let window = web_sys::window().unwrap();
        if let Ok(game_client) = js_sys::Reflect::get(&window, &"gameClient".into()) {
            if let Ok(players_json) = serde_json::to_string(players) {
                if let Ok(update_fn) = js_sys::Reflect::get(&game_client, &"update_all_players".into()) {
                    if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                        let args = js_sys::Array::new();
                        args.push(&players_json.into());
                        let _ = func.apply(&game_client, &args);
                    }
                }
            }
        }
    }

    fn update_position_display(username: &str, x: u32, y: u32) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        // Update position display if it's the current user
        if let Some(user_display) = document.get_element_by_id("userDisplay") {
            if let Some(current_user) = user_display.text_content() {
                if current_user == username {
                    if let Some(pos_display) = document.get_element_by_id("positionDisplay") {
                        pos_display.set_text_content(Some(&format!("Position: ({}, {})", x, y)));
                    }
                }
            }
        }
    }

    fn handle_ping_response() {
        let window = web_sys::window().unwrap();
        if let Ok(callback) = js_sys::Reflect::get(&window, &"onPingReceived".into()) {
            if let Ok(func) = callback.dyn_into::<js_sys::Function>() {
                let _ = func.call0(&window);
            }
        }
    }

    fn handle_chat_message(chat_msg: ChatMessage, pending_messages: &Rc<RefCell<HashMap<String, web_sys::Element>>>) {
        let formatted_timestamp = Self::format_timestamp(&chat_msg.timestamp);
        let formatted_message = format!("[{}] {}: {}", formatted_timestamp, chat_msg.username, chat_msg.message);
        
        // Remove from pending if it's our message
        let message_key = chat_msg.message.to_lowercase().trim().to_string();
        let mut pending = pending_messages.borrow_mut();
        if let Some(pending_element) = pending.remove(&message_key) {
            if let Some(parent) = pending_element.parent_node() {
                let _ = parent.remove_child(&pending_element);
            }
        }
        
        Self::append_chat_message(&formatted_message);
    }

    fn add_pending_message(message: &str, pending_messages: &Rc<RefCell<HashMap<String, web_sys::Element>>>) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(chat_messages) = document.get_element_by_id("chatMessages") {
            let message_div = document.create_element("div").unwrap();
            
            // Format like server messages
            let timestamp = Self::format_current_timestamp();
            let username = document.get_element_by_id("userDisplay")
                .and_then(|el| el.text_content())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let formatted_text = format!("[{}] {}: {} [SENDING...]", timestamp, username, message);
            message_div.set_text_content(Some(&formatted_text));
            message_div.set_attribute("style", "opacity: 0.6; font-style: italic").unwrap();
            
            let _ = chat_messages.append_child(&message_div);
            chat_messages.set_scroll_top(chat_messages.scroll_height());
            
            // Store in pending
            let message_key = message.to_lowercase().trim().to_string();
            pending_messages.borrow_mut().insert(message_key, message_div);
        }
    }

    fn append_chat_message(message: &str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(chat_messages) = document.get_element_by_id("chatMessages") {
            let message_div = document.create_element("div").unwrap();
            message_div.set_text_content(Some(message));
            
            let _ = chat_messages.append_child(&message_div);
            chat_messages.set_scroll_top(chat_messages.scroll_height());
            
            // Limit messages to 100 - use child_element_count for counting
            while chat_messages.child_element_count() > 100 {
                if let Some(first_child) = chat_messages.first_child() {
                    let _ = chat_messages.remove_child(&first_child);
                }
            }
        }
    }

    fn format_timestamp(timestamp_value: &serde_json::Value) -> String {
        if let Some(timestamp_str) = timestamp_value.as_str() {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                let utc_time = parsed.with_timezone(&chrono::Utc);
                return format!("{:02}:{:02}:{:02}.{:02}", 
                    utc_time.hour(), 
                    utc_time.minute(), 
                    utc_time.second(),
                    utc_time.nanosecond() / 10_000_000
                );
            }
        }
        Self::format_current_timestamp()
    }

    fn format_current_timestamp() -> String {
        let now = chrono::Utc::now();
        format!("{:02}:{:02}:{:02}.{:02}", 
            now.hour(), 
            now.minute(), 
            now.second(),
            now.nanosecond() / 10_000_000
        )
    }

    fn is_websocket_connected(&self) -> bool {
        if let Some(ref websocket) = self.websocket {
            websocket.ready_state() == WebSocket::OPEN
        } else {
            false
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("🚀 IronVein Rust Game Engine initialized!");
} 