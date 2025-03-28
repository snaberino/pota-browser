use std::net::TcpStream;
use std::sync::Arc;

use reqwest::Error;

use tokio::task;
use tokio::sync::{ mpsc, Mutex };

// use tokio::sync::Mutex;

use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};

use crate::chromium::chromium::ChromiumProfile;
use crate::chromium::driver::Driver;



pub fn cdp_main(profile: ChromiumProfile) {
    let profile_clone_for_cdp = profile.clone();

    task::spawn(async move {
        let socket = Arc::new(Mutex::new(get_socket(profile_clone_for_cdp.clone()).await.unwrap()));

        // Avvia un task separato per gestire il loop continuo di cdp_mit
        let profile_clone_for_mit = profile_clone_for_cdp.clone();
        let socket_clone_for_mit = socket.clone();

        task::spawn(async move {
            if let Err(e) = cdp_mit(profile_clone_for_mit, socket_clone_for_mit).await {
                eprintln!("Error while opening Chrome DevTools Protocol: {}", e);
            }
        });

        // Avvia un task separato per navigare a un sito web
        let mut driver = Driver::new(socket.clone(), profile_clone_for_cdp.clone());

        // Avvia un task separato per navigare a un sito web
        // let driver_clone_for_navigation = driver.clone();
        task::spawn(async move {
            if let Err(e) = navigate_to_url(&mut driver, "https://www.yandex.com").await {
                eprintln!("Error while navigating: {}", e);
            }
        });
    });
}

async fn navigate_to_url(
    driver: &mut Driver,
    url: &str,
) -> Result<(), Error> {
    // Comando per navigare a un URL

    // Invia il comando tramite il driver
    driver.navigate_to(url).await;
    println!("Navigated to URL: {}", url);

    Ok(())
}


// This is the man in-the-middle function that will handle the communication with the Chrome DevTools Protocol
// It will listen for incoming messages and handle them accordingly.
async fn cdp_mit(
    profile: ChromiumProfile,
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
) -> Result<(), Error> {

    let mut driver = Driver::new(socket.clone(), profile.clone());

    driver.enable_page_domain().await;
    driver.enable_fetch_domain().await;
    driver.navigate_to("https://ipscore.io").await; // I need to navigate to a page in order to trigger proxy authentication. When one authentication is made, the rest of the requests are authenticated automatically.

    // Listen indefinitely for incoming messages
    // We then handle the Fetch.authRequired event, which is triggered when a request requires authentication.

    loop {
        let msg = {
            let mut socket_guard = socket.lock().await;
            socket_guard.read().expect("Error while reading incoming msg")
        };
        if let Message::Text(text) = msg {
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            println!("Event: {}", response); // Debugging
            // println!("Event: {}", response["result"]); // Debugging
    
            match response["method"].as_str() {
                Some("Page.loadEventFired") => { println!("Page loaded notification sent") }, // Notify the bot that the page has loaded
                // Some("Fetch.authRequired") => handle_auth_required(&mut *socket.lock().await, &response, &profile),
                Some("Fetch.authRequired") => { println!("AUTHHHHHHHHHHHHSHDF"); driver.handle_auth_required(&response).await; },
                Some("Fetch.requestPaused") => driver.handle_request_paused(&response).await,
                Some("Runtime.evaluate") => println!("Event Runtime.evaluate received"), // Debugging
                _ => { println!("Unhandled event: {}", response); }, // Debugging
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
