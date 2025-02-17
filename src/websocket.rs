use std::net::TcpStream;

use reqwest::Error;
use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};
use serde_json::json;

use crate::chrome::ChromeProfile;
// use url::Url;

// Function to create a WebSocket connection to the Chrome DevTools Protocol given a Chrome profile configuration.
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
            "url": "https://www.browserscan.net"
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
                        "requestId": response["params"]["requestId"]
                    }
                });
                socket.send(Message::Text(continue_request.to_string().into())).unwrap();
            }
        }
    }
    Ok(())
}



pub fn set_timezone_cdp(profile: &ChromeProfile) -> Result<(), Error> {

    let mut socket = get_ws_url(&profile).unwrap();
    // let set_timezone_cmd = json!({
    //     "id": 1,
    //     "method": "Emulation.setTimezoneOverride",
    //     "params": {
    //         "timezoneId": "America/Los_Angeles" // Imposta il fuso orario desiderato
    //     }
    // });
    // socket.send(Message::Text(set_timezone_cmd.to_string().into())).unwrap();

    // Ok(())
    


    // Script che sovrascrive getTimezoneOffset per restituire, ad esempio, -60 (UTC+1)
    let script = r#"
        (function() {
            const originalGetTimezoneOffset = Date.prototype.getTimezoneOffset;
            Date.prototype.getTimezoneOffset = function() {
                return -120;
            };
        })();
    "#;

    // Usa serde_json per serializzare lo script in modo sicuro
    let params = json!({ "source": script });
    let command = json!({
        "id": 1,
        "method": "Page.addScriptToEvaluateOnNewDocument",
        "params": params,
    });

    // Invia il comando al CDP
    socket.send(Message::Text(command.to_string().into())).unwrap();

    // (Opzionale) Leggi la risposta
    if let Ok(msg) = socket.read() {
        println!("Risposta: {}", msg);
    }
    Ok(())

}