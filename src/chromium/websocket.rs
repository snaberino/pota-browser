use std::net::TcpStream;
use std::sync::Arc;

use reqwest::Error;
use tokio::task;
use tokio::sync::{ mpsc, Mutex };

use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};

use crate::chromium::chromium::ChromiumProfile;
use crate::chromium::driver::Driver;

pub fn cdp_main(profile: ChromiumProfile) {
    let profile_clone_for_cdp = profile.clone();

    task::spawn(async move {
        let socket = Arc::new(Mutex::new(get_socket(profile_clone_for_cdp.clone()).await.unwrap()));

        // Create a shared driver instance
        let driver = Arc::new(Mutex::new(Driver::new(socket.clone(), profile_clone_for_cdp.clone())));

        // Task that will handle the MIT for the Chrome DevTools Protocol
        let profile_clone_for_mit = profile_clone_for_cdp.clone();
        let socket_clone_for_mit = socket.clone();
        let driver_clone_for_mit = driver.clone();

        task::spawn(async move {
            if let Err(e) = cdp_mit(profile_clone_for_mit, socket_clone_for_mit, driver_clone_for_mit).await {
                eprintln!("Error while opening Chrome DevTools Protocol: {}", e);
            }
        });

    });
}

async fn navigate_to_url(driver: Arc<Mutex<Driver>>, url: &str) -> Result<(), Error> {
    let mut driver_guard = driver.lock().await;
    driver_guard.navigate_to(url).await;
    println!("Navigated to URL: {}", url);

    Ok(())
}

async fn cdp_mit(
    profile: ChromiumProfile,
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
    driver: Arc<Mutex<Driver>>,
) -> Result<(), Error> {
    {
        let mut driver_guard = driver.lock().await;
        driver_guard.enable_page_domain().await;
        driver_guard.enable_fetch_domain().await;
        driver_guard.navigate_to("https://ipscore.io").await;
    }

    loop {
        let msg = {
            let mut socket_guard = socket.lock().await;
            socket_guard.read().expect("Error while reading incoming msg")
        };
        if let Message::Text(text) = msg {
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            println!("Event: {}", response);
            match response["method"].as_str() {
                Some("Page.frameStoppedLoading") => {
                    println!("Page loaded notification sent");
                    // AUTOMATION START IF NEEDED

                }
                Some("Fetch.authRequired") => {
                    let mut driver_guard = driver.lock().await;
                    driver_guard.handle_auth_required(&response).await;
                }
                Some("Fetch.requestPaused") => {
                    let mut driver_guard = driver.lock().await;
                    driver_guard.handle_request_paused(&response).await;
                }
                _ => {
                    println!("Unhandled event: {}", response);
                }
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
    let (socket, _) = connect(websocket_url).expect("Get socket error.");
    Ok(socket)
}
