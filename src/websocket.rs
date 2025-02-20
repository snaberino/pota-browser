use std::net::TcpStream;
use reqwest::Error;

use tokio::task;

use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};
use serde_json::json;

use crate::chrome::ChromeProfile;

// Function to create a WebSocket connection to the Chrome DevTools Protocol given a Chrome profile configuration.
fn get_socket(profile: &ChromeProfile) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
    // DevTools URL
    let devtools_url = format!("http://localhost:{}/json", profile.debugging_port);

    // Retrieve the list of pages
    let response = reqwest::blocking::get(devtools_url);
    let pages: Vec<serde_json::Value> = response.unwrap().json().unwrap();

    // Retrieve the WebSocket URL of the first page
    let websocket_url = pages[0]["webSocketDebuggerUrl"].as_str().unwrap();
    println!("WebSocket URL: {}", websocket_url);

    // Perform the WebSocket connection
    let (socket, _) = connect(websocket_url)
        .expect("Errore di connessione");

    // socket.close(None).expect("Errore nella chiusura della connessione WebSocket");
    Ok(socket)
}

pub fn set_proxy_cdp(profile: &ChromeProfile) -> Result<(), Error> {
    let mut socket = get_socket(&profile).unwrap();

    // Enable the Fetch domain in order to handle requests
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

    // I'll navigate to a website in order to trigger the proxy authentication request
    let navigate_cmd = json!({
        "id": 3,
        "method": "Page.navigate",
        "params": {
            // "url": "https://abrahamjuliot.github.io/creepjs/"
            // "url": "https://ipscore.io/"
            // "url": "https://www.browserscan.net"
            "url": "https://browserleaks.com/client-hints"
        }
    });
    socket.send(Message::Text(navigate_cmd.to_string().into())).unwrap();

    // Handle the Fetch.authRequired event, will loop until the event is received and then send the credentials.
    loop {
        let msg = socket.read().expect("Errore nella lettura del messaggio");
        if let Message::Text(text) = msg {
            println!("Messaggio ricevuto: {}", text);
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            if response["method"] == "Fetch.authRequired" {
                println!("Evento Fetch.authRequired ricevuto");
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

                break;

            } else if response["method"] == "Fetch.requestPaused" {
                println!("Evento Fetch.requestPaused ricevuto");
                let continue_request = json!({
                    "id": 4,
                    "method": "Fetch.continueRequest",
                    "params": {
                        "requestId": response["params"]["requestId"],
                    }
                });
                socket.send(Message::Text(continue_request.to_string().into())).unwrap();
            }
        }
    }
    Ok(())
}

pub fn set_timezone_cdp(profile: &ChromeProfile) -> Result<(), Error> {

    let mut socket = get_socket(&profile).unwrap();
    let set_timezone_cmd = json!({
        "id": 1,
        "method": "Emulation.setTimezoneOverride",
        "params": {
            "timezoneId": "Europe/Madrid" // Imposta il fuso orario desiderato
        }
    });
    socket.send(Message::Text(set_timezone_cmd.to_string().into())).unwrap();
    // Read the response message
    let msg = socket.read().expect("Errore nella lettura del messaggio");
    if let Message::Text(text) = msg {
        println!("Messaggio ricevuto: {}", text);
    }


    // Inject the JavaScript code
    let inject_js_cmd = json!({
        "id": 2,
        "method": "Page.addScriptToEvaluateOnNewDocument",
        "params": {
            "source": r#"
                (function() {
                    // Save the original method
                    const originalResolvedOptions = Intl.DateTimeFormat.prototype.resolvedOptions;
                    
                    Intl.DateTimeFormat.prototype.resolvedOptions = function() {
                        // Get the original options
                        const options = originalResolvedOptions.apply(this, arguments);
                        // Force the timeZone value
                        options.timeZone = 'Europe/Madrid';
                        return options;
                    };
                })();
            "#
        }
    });
    socket.send(Message::Text(inject_js_cmd.to_string().into())).unwrap();

    // Read the response message for the injected JS
    let msg = socket.read().expect("Errore nella lettura del messaggio");
    if let Message::Text(text) = msg {
        println!("Messaggio ricevuto: {}", text);
    }


    Ok(())

}








async fn get_socket1(profile: ChromeProfile) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
    // DevTools URL
    let devtools_url = format!("http://localhost:{}/json", profile.debugging_port);

    // Retrieve the list of pages
    let response = reqwest::get(&devtools_url).await?;
    let pages: Vec<serde_json::Value> = response.json().await?;

    // Retrieve the WebSocket URL of the first page
    let websocket_url = pages[0]["webSocketDebuggerUrl"].as_str().unwrap();
    println!("WebSocket URL: {}", websocket_url);
    let (socket, _) = connect(websocket_url).expect("errore");
    Ok(socket)
}

async fn start_cdp(profile: ChromeProfile) -> Result<(), Error> {
    let mut socket = get_socket1(profile).await.unwrap();
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
        "id": 3,
        "method": "Page.navigate",
        "params": {
            "url": "https://browserleaks.com/client-hints"
        }
    });
    socket.send(Message::Text(navigate_cmd.to_string().into())).unwrap();

    // Listen indefinitely for incoming messages
    loop {
        let msg = socket.read().expect("Errore nella lettura del messaggio");
        if let Message::Text(text) = msg {
            println!("Messaggio ricevuto: {}", text);
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            if response["method"] == "Fetch.authRequired" {
                println!("Evento Fetch.authRequired ricevuto");
                // let auth_challenge_response = json!({
                //     "id": 2,
                //     "method": "Fetch.continueWithAuth",
                //     "params": {
                //         "requestId": response["params"]["requestId"],
                //         "authChallengeResponse": {
                //             "response": "ProvideCredentials",
                //             "username": profile.proxy.proxy_username.clone(),
                //             "password": profile.proxy.proxy_password.clone()
                //         }
                //     }
                // });
                // socket.send(Message::Text(auth_challenge_response.to_string().into())).unwrap();
            } else if response["method"] == "Fetch.requestPaused" {
                println!("Evento Fetch.requestPaused ricevuto");
                let continue_request = json!({
                    "id": 4,
                    "method": "Fetch.continueRequest",
                    "params": {
                        "requestId": response["params"]["requestId"],
                        "headers": [
                            { "name": "Accept-Encoding", "value": "br, gzip, deflate" },
                            { "name": "Sec-CH-UA", "value": "\"Chromium\";v=\"116\", \"Google Chrome\";v=\"116\", \";Not A Brand\";v=\"99\"" },
                            { "name": "Sec-CH-UA-Platform", "value": "\"MacOS\"" },
                          ]
                    }
                });
                socket.send(Message::Text(continue_request.to_string().into())).unwrap();
            }
        }
    }

    // Ok(())
}

pub fn start_cdp_listener(profile: ChromeProfile) {
    task::spawn(async move {
        if let Err(e) = start_cdp(profile).await {
            eprintln!("Errore durante l'ascolto del Chrome DevTools Protocol: {}", e);
        }
    });
}