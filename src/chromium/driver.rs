use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::Mutex;

use tungstenite::{WebSocket, stream::MaybeTlsStream, Message};

use serde_json::json;

use crate::chromium::chromium::ChromiumProfile;

// Global counter for generating unique IDs
static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

// Function to generate unique IDs for CDP messages
fn get_next_id() -> u64 {
    ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub struct Driver {
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
    profile: ChromiumProfile,
}

impl Driver {
    pub fn new(
        socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
        profile: ChromiumProfile,
    ) -> Self {
        Self { socket, profile }
    }

    pub async fn enable_page_domain(&mut self) {
        let enable_page_domain = json!({
            "id": get_next_id(),
            "method": "Page.enable"
        });
        let mut socket_guard = self.socket.lock().await;
        socket_guard
            .send(Message::Text(enable_page_domain.to_string().into()))
            .unwrap();
        println!("Page domain enabled"); // Debug
    }

    pub async fn enable_fetch_domain(&mut self) {
        let enable_fetch_domain = json!({
            "id": get_next_id(),
            "method": "Fetch.enable",
            "params": {
                "patterns": [{
                    "urlPattern": "*",
                    "requestStage": "Request",
                }],
                "handleAuthRequests": true
            }
        });
        let mut socket_guard = self.socket.lock().await;
        socket_guard
            .send(Message::Text(enable_fetch_domain.to_string().into()))
            .unwrap();
        println!("Fetch domain enabled"); // Debug
    }

    pub async fn handle_auth_required(&mut self, response: &serde_json::Value) {
        println!("Auth required");
        let auth_challenge_response = json!({
            "id": get_next_id(),
            "method": "Fetch.continueWithAuth",
            "params": {
                "requestId": response["params"]["requestId"],
                "authChallengeResponse": {
                    "response": "ProvideCredentials",
                    "username": self.profile.proxy.proxy_username.clone(),
                    "password": self.profile.proxy.proxy_password.clone()
                }
            }
        });
        let mut socket_guard = self.socket.lock().await;
        socket_guard
            .send(Message::Text(auth_challenge_response.to_string().into()))
            .unwrap();
    }

    pub async fn handle_request_paused(&mut self, response: &serde_json::Value) {
        let resource_type = response["params"]["resourceType"].as_str().unwrap_or("");
        let url = response["params"]["request"]["url"].as_str().unwrap_or("");

        if self.profile.images == 0 && Self::is_image_request(resource_type, url) {
            self.block_request(response).await;
        } else if self.profile.images == 2 && Self::is_non_captcha_image(resource_type, url) {
            self.block_request(response).await;
        } else {
            self.continue_request(response).await;
        }
        println!("handle request_paused called"); // Debugging
    }

    pub async fn navigate_to(&mut self, url: &str) {
        let navigate_cmd = json!({
            "id": get_next_id(),
            "method": "Page.navigate",
            "params": {
                "url": url
            }
        });
        let mut socket_guard = self.socket.lock().await;
        socket_guard
            .send(Message::Text(navigate_cmd.to_string().into()))
            .unwrap();
        println!("Navigation command sent to URL: {}", url); // Debugging
    }



    async fn block_request(&mut self, response: &serde_json::Value) {
        let fail_request = json!({
            "id": get_next_id(),
            "method": "Fetch.failRequest",
            "params": {
                "requestId": response["params"]["requestId"],
                "errorReason": "BlockedByClient"
            }
        });
        let mut socket_guard = self.socket.lock().await;
        socket_guard
            .send(Message::Text(fail_request.to_string().into()))
            .unwrap();
    }

    async fn continue_request(&mut self, response: &serde_json::Value) {
        let continue_request = json!({
            "id": get_next_id(),
            "method": "Fetch.continueRequest",
            "params": {
                "requestId": response["params"]["requestId"]
            }
        });
        let mut socket_guard = self.socket.lock().await;
        socket_guard
            .send(Message::Text(continue_request.to_string().into()))
            .unwrap();
    }



    fn is_image_request(resource_type: &str, url: &str) -> bool {
        resource_type == "Image" || url.ends_with(".jpg") || url.ends_with(".png") || url.ends_with(".gif")
    }
    
    fn is_non_captcha_image(resource_type: &str, url: &str) -> bool {
        let captcha_providers = vec!["recaptcha.net", "hcaptcha.com", "google.com/recaptcha"];
        resource_type == "Image" && !captcha_providers.iter().any(|provider| url.contains(provider))
    }
    
}


