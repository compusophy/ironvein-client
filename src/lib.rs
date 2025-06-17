use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub struct Client {
    server_url: String,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Client {
        Client {
            server_url: "http://localhost:8080".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn set_server_url(&mut self, url: String) {
        self.server_url = url;
        console_log!("Server URL set to: {}", self.server_url);
    }

    #[wasm_bindgen]
    pub fn get_server_url(&self) -> String {
        self.server_url.clone()
    }

    #[wasm_bindgen]
    pub async fn fetch_data(&self) -> Result<String, JsValue> {
        let window = web_sys::window().unwrap();
        let url = format!("{}/api/data", self.server_url);
        
        let request = web_sys::Request::new_with_str(&url)?;
        request.headers().set("Content-Type", "application/json")?;

        let response = wasm_bindgen_futures::JsFuture::from(
            window.fetch_with_request(&request)
        ).await?;

        let response: web_sys::Response = response.dyn_into()?;
        let text = wasm_bindgen_futures::JsFuture::from(
            response.text()?
        ).await?;

        Ok(text.as_string().unwrap_or_default())
    }

    #[wasm_bindgen]
    pub fn greet(&self, name: &str) -> String {
        format!("Hello, {}! This is the IronVein client.", name)
    }
}

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("IronVein Client WASM module initialized!");
} 