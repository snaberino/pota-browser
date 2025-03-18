use std::net::TcpStream;
use reqwest::Error;

use tokio::task;

use tungstenite::{connect, WebSocket, Message, stream::MaybeTlsStream};
use serde_json::json;

use crate::chromium::ChromeProfile;

async fn get_socket1(profile: ChromeProfile) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
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

// We now start a process that listens for incoming messages and handles them accordingly.
// Using Chrome DevTools Protocol, we enable the Fetch domain in order to fetch every request made by the browser.
// We then listen for the Fetch.authRequired event, which is triggered when a request requires authentication.
// We send the credentials to the browser, and the request is then allowed to proceed.
// We also listen for the Fetch.requestPaused event, which is triggered when a request is paused, so this happen for every requests.
// We then continue the request by sending the necessary spoofed headers.
async fn start_cdp(profile: ChromeProfile) -> Result<(), Error> {
    let mut socket = get_socket1(profile.clone()).await.unwrap();
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
            "url": "https://browserleaks.com/client-hints"
        }
    });
    socket.send(Message::Text(navigate_cmd.to_string().into())).unwrap();

    // Listen indefinitely for incoming messages
    loop {
        let msg = socket.read().expect("Error while reading incoming msg");
        if let Message::Text(text) = msg {
            //println!("Message  recevied: {}", text); //debugging
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            println!("Event: {}", response); //debugging
            //println!("Method: {}", response["method"]); //debugging
            // Section for handling all the fetch events
            // Handle the Fetch.authRequired event required for proxy authentication
            if response["method"] == "Fetch.authRequired" {
                println!("Evento Fetch.authRequired ricevuto"); //debugging
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
            // That section handle the Fetch.requestPaused event that occur for every request, here i can set custom headers spoofing the Client Hints https://browserleaks.com/client-hints
            // So for every request i must set concrete headers related to the whole chrome config
            } else if response["method"] == "Fetch.requestPaused" {
                //println!("Evento Fetch.requestPaused ricevuto"); //debugging
                let continue_request = json!({
                    "id": 4,
                    "method": "Fetch.continueRequest",
                    "params": {
                        "requestId": response["params"]["requestId"],
                        "headers" : [
                            {
                                "name": "sec-ch-ua",
                                "value": "\"Chromium\";v=\"134\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"134\""
                            },
                            {
                                "name": "sec-ch-ua-platform-version",
                                "value": "19.0.0"
                            },
                            {
                                "name": "Sec-CH-UA-Platform",
                                "value": "\"Windows\""
                            },
                            {
                                "name": "Sec-CH-UA-Platform",
                                "value": "Chromium;v=134, Not:A-Brand;v=24, Google Chrome;v=134"
                            },
                            {
                                "name": "Sec-CH-UA-Platform-Version",
                                "value": "19.0.0"
                            },
                            {
                                "name": "Sec-CH-UA-Mobile",
                                "value": "?0"
                            },
                            {
                                "name": "Sec-CH-UA-Full-Version",
                                "value": "134.0.6998.36"
                            },
                            {
                                "name": "Sec-CH-UA-Full-Version-List",
                                "value": "Chromium;v=134.0.6998.36, Not:A-Brand;v=24.0.0.0, Google Chrome;v=134.0.6998.36"
                            },
                            {
                                "name": "Sec-CH-UA-Platform",
                                "value": "Windows"
                            },
                            {
                                "name": "Sec-CH-UA-Arch",
                                "value": "x86"
                            },
                            {
                                "name": "Sec-CH-UA-Bitness",
                                "value": "64"
                            },
                            {
                                "name": "Sec-CH-UA-WoW64",
                                "value": "?0"
                            },
                            {
                                "name": "Sec-CH-UA-Model",
                                "value": ""
                            },
                            {
                                "name": "Sec-CH-UA-Form-Factors",
                                "value": "Desktop"
                            },
                            {
                                "name": "Sec-CH-Lang",
                                "value": "not received"
                            },
                            {
                                "name": "Sec-CH-Save-Data",
                                "value": "not received"
                            },
                            {
                                "name": "Sec-CH-Width",
                                "value": "not received"
                            },
                            {
                                "name": "Sec-CH-Viewport-Width",
                                "value": "1280"
                            },
                            {
                                "name": "Viewport-Width",
                                "value": "1280"
                            },
                            {
                                "name": "Sec-CH-Viewport-Height",
                                "value": "585"
                            },
                            {
                                "name": "Sec-CH-DPR",
                                "value": "1"
                            },
                            {
                                "name": "DPR",
                                "value": "1"
                            },
                            {
                                "name": "Sec-CH-Device-Memory",
                                "value": "8"
                            },
                            {
                                "name": "Device-Memory",
                                "value": "8"
                            },
                            {
                                "name": "RTT",
                                "value": "100"
                            },
                            {
                                "name": "Downlink",
                                "value": "1.55"
                            },
                            {
                                "name": "ECT",
                                "value": "4g"
                            },
                            {
                                "name": "Sec-CH-Prefers-Color-Scheme",
                                "value": "light"
                            },
                            {
                                "name": "Sec-CH-Prefers-Reduced-Motion",
                                "value": "no-preference"
                            },
                            {
                                "name": "Sec-CH-Prefers-Reduced-Transparency",
                                "value": "no-preference"
                            },
                            {
                                "name": "Sec-CH-Prefers-Contrast",
                                "value": "not received"
                            },
                            {
                                "name": "Sec-CH-Forced-Colors",
                                "value": "not received"
                            }
                        ]
                    }
                });
                socket.send(Message::Text(continue_request.to_string().into())).unwrap();
            } else if response["method"] == "Runtime.evaluate" {
                println!("Event Runtime.evaluate receveid"); //debugging
            }
        }
    }
}

pub fn start_cdp_listener(profile: ChromeProfile) {
    task::spawn(async move {
        if let Err(e) = start_cdp(profile).await {
            eprintln!("Error while opening Chrome DevTools Protocol: {}", e);
        }
    });
}