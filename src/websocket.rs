use std::net::TcpStream;

use reqwest::Error;
use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};
use serde_json::json;

use crate::chrome::ChromeProfile;
// use url::Url;

// Function to get the WebSocket URL of the Chrome DevTools Protocol, given a specific Chrome profile
pub fn get_ws_url(profile: &ChromeProfile) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
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

    let mut socket = get_ws_url(&profile).unwrap();
    // Abilita l'intercettazione delle richieste di rete per injectare l'autenticazione
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

    // Comando per navigare a una pagina
    let navigate_cmd = json!({
        "id": 3,
        "method": "Page.navigate",
        "params": {
            // "url": "https://abrahamjuliot.github.io/creepjs/"
            // "url": "https://ipscore.io/"
            "url": "https://www.browserscan.net"
        }
    });
    socket.send(Message::Text(navigate_cmd.to_string().into())).unwrap();

    // Gestisci l'evento di autenticazione richiesta
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
                        "requestId": response["params"]["requestId"]
                    }
                });
                socket.send(Message::Text(continue_request.to_string().into())).unwrap();
            }
        }
    }
    Ok(())
}