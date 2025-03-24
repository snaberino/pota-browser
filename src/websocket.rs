use std::net::TcpStream;
use reqwest::Error;

use tokio::task;

use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};
use serde_json::json;

use crate::chromium::ChromiumProfile;

pub fn start_cdp_listener(profile: ChromiumProfile) {
    let profile_clone_for_cdp = profile.clone();
    task::spawn(async move {
        if let Err(e) = start_cdp(profile_clone_for_cdp).await {
            eprintln!("Error while opening Chrome DevTools Protocol: {}", e);
        }
    });

    let profile_clone_for_bot = profile.clone();
    task::spawn(async move {
        if let Err(e) = test_bot(profile_clone_for_bot).await {
            eprintln!("Error while testing bot: {}", e);
        }
    });
}

// We now start a process that listens for incoming messages and handles them accordingly.
// Using Chrome DevTools Protocol, we enable the Fetch domain in order to fetch every request made by the browser.
// We then listen for the Fetch.authRequired event, which is triggered when a request requires authentication.
// We send the credentials to the browser, and the request is then allowed to proceed.
// We also listen for the Fetch.requestPaused event, which is triggered when a request is paused, so this happen for every requests.
// We then continue the request by sending the necessary spoofed headers.
async fn start_cdp(profile: ChromiumProfile) -> Result<(), Error> {
    let mut socket = get_socket(profile.clone()).await.unwrap();
    
    let enable_fetch_cmd = json!({
        "id": 1,
        "method": "Fetch.enable",
        "params": {
            "patterns": [{
                "urlPattern": "*",
                "requestStage": "Request",
            }],
            "handleAuthRequests": true
        }
    });
    socket.send(Message::Text(enable_fetch_cmd.to_string().into())).unwrap();

    // Listen indefinitely for incoming messages
    // We then handle the Fetch.authRequired event, which is triggered when a request requires authentication.

    loop {
        let msg = socket.read().expect("Error while reading incoming msg");
        if let Message::Text(text) = msg {
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            println!("Event: {}", response); // Debugging
    
            match response["method"].as_str() {
                Some("Fetch.authRequired") => handle_auth_required(&mut socket, &response, &profile),
                Some("Fetch.requestPaused") => handle_request_paused(&mut socket, &response, &profile),
                Some("Runtime.evaluate") => println!("Event Runtime.evaluate received"), // Debugging
                _ => println!("Unhandled event: {}", response["method"]), // Debugging
            }
        }
    }
}

async fn get_socket(profile: ChromiumProfile) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
    // DevTools URL
    let devtools_url = format!("http://127.0.0.1:{}/json", profile.debugging_port);

    // Retrieve the list of pages
    let response = reqwest::get(&devtools_url).await?;
    let pages: Vec<serde_json::Value> = response.json().await?;

    // Retrieve the WebSocket URL of the first page
    let websocket_url = pages[0]["webSocketDebuggerUrl"].as_str().unwrap();
    println!("WebSocket URL: {}", websocket_url);
    let (socket, _) = connect(websocket_url).expect("errore");
    Ok(socket)
}


// Event handlers, maybe we can move them to a separate module for CDP events handling
// Fetch.authRequired event handler
fn handle_auth_required(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, response: &serde_json::Value, profile: &ChromiumProfile) {
    println!("Evento Fetch.authRequired ricevuto"); // Debugging
    let auth_challenge_response = json!({
        "id": 2,
        "method": "Fetch.continueWithAuth",
        "params": {
            "requestId": response["params"]["requestId"],
            "authChallengeResponse": {
                "response": "ProvideCredentials",
                "username": profile.proxy.proxy_username.clone(),
                "password": profile.proxy.proxy_password.clone()
            }
        }
    });
    socket.send(Message::Text(auth_challenge_response.to_string().into())).unwrap();
}

// Fetch.requestPaused event handler
fn handle_request_paused(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, response: &serde_json::Value, profile: &ChromiumProfile) {
    let resource_type = response["params"]["resourceType"].as_str().unwrap_or("");
    let url = response["params"]["request"]["url"].as_str().unwrap_or("");

    if profile.images == 0 && is_image_request(resource_type, url) { // Block all images
        block_request(socket, response);
    } else if profile.images == 2 && is_non_captcha_image(resource_type, url) { // Block all images except CAPTCHA images
        block_request(socket, response);
        println!("Blocked non-CAPTCHA image request: {}", url); // Debugging
    } else {
        continue_request(socket, response);
    }
}

// Fetch.failRequest method to block a request
fn block_request(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, response: &serde_json::Value) {
    let fail_request = json!({
        "id": 4,
        "method": "Fetch.failRequest",
        "params": {
            "requestId": response["params"]["requestId"],
            "errorReason": "BlockedByClient"
        }
    });
    socket.send(Message::Text(fail_request.to_string().into())).unwrap();
}

// Fetch.continueRequest method to continue a request
fn continue_request(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, response: &serde_json::Value) {
    let continue_request = json!({
        "id": 3,
        "method": "Fetch.continueRequest",
        "params": {
            "requestId": response["params"]["requestId"]
        }
    });
    socket.send(Message::Text(continue_request.to_string().into())).unwrap();
}

// Check if a request is an image request
fn is_image_request(resource_type: &str, url: &str) -> bool {
    resource_type == "Image" || url.ends_with(".jpg") || url.ends_with(".png") || url.ends_with(".gif")
}

// Check if a request is a non-CAPTCHA image request
fn is_non_captcha_image(resource_type: &str, url: &str) -> bool {
    let captcha_providers = vec!["recaptcha.net", "hcaptcha.com", "google.com/recaptcha"];
    resource_type == "Image" && !captcha_providers.iter().any(|provider| url.contains(provider))
}



// DEBUGGING
// TESTING AUTOMATION

async fn test_bot(profile: ChromiumProfile) -> Result<(), Error> {
    println!("Testing bot"); // debugging
    let mut socket = get_socket(profile.clone()).await.unwrap();

    let enable_fetch_cmd = json!({
        "id": 1,
        "method": "Fetch.enable",
        "params": {
            "patterns": [{
                "urlPattern": "*",
                "requestStage": "Request",
            }],
            "handleAuthRequests": true
        }
    });
    socket.send(Message::Text(enable_fetch_cmd.to_string().into())).unwrap();

    let navigate_cmd = json!( {
        "id": 2,
        "method": "Page.navigate",
        "params": {
            "url": "https://www.google.com/recaptcha/api2/demo"
        }
    });
    socket.send(Message::Text(navigate_cmd.to_string().into())).unwrap();
    println!("message sent"); // debugging
    Ok(())
}