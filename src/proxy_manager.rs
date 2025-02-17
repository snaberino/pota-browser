// use reqwest::Client;
use reqwest::Client;

use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fs::File;
use std::io::{ Write, Read };
use std::io::BufReader;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ProxyConfig {
    pub proxy_name: String,
    pub proxy_type: String,
    pub proxy_host: String,
    pub proxy_port: String,
    pub proxy_username: String,
    pub proxy_password: String,
}

pub type ProxiesConfig = Vec<ProxyConfig>;

pub fn load_proxy_configs() -> ProxiesConfig {
    let file_path = "proxy_config.json";
    let file = File::open(file_path).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).expect("Unable to read file");

    // If the file is empty, return an empty vector
    if content.trim().is_empty() {
        let empty_proxies: ProxiesConfig = vec![];
        save_proxy_configs(&empty_proxies);
        empty_proxies
    } else {
        serde_json::from_str(&content).expect("Unable to parse JSON")
    }
}

pub fn save_proxy_configs(proxy_configs: &ProxiesConfig) {
    let json = serde_json::to_string_pretty(proxy_configs).expect("Unable to serialize proxy configs");
    let mut file = File::create("proxy_config.json").expect("Unable to create file");
    file.write_all(json.as_bytes()).expect("Unable to write data");
}

// Function to test a proxy, need to improve this function
pub async fn test_proxy(proxy_url: &str) -> Result<(String, String), String> {
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(proxy_url).map_err(|e| e.to_string())?)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get("https://ipinfo.io/json").send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>().await {
                let ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
                let country = json["country"].as_str().unwrap_or("Unknown Country").to_string();
                Ok((ip, country))
            } else {
                Err("Errore nel parsing della risposta.".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

// pub fn test_proxy1(proxy_url: &str) -> Result<(String, String), String> {
//     let client = Client::builder()
//         .proxy(reqwest::Proxy::all(proxy_url).map_err(|e| e.to_string())?)
//         .timeout(Duration::from_secs(10))
//         .build()
//         .map_err(|e| e.to_string())?;

//     match client.get("https://ipinfo.io/json").send() {
//         Ok(response) => {
//             if let Ok(json) = response.json::<Value>() {
//                 let ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
//                 let country = json["country"].as_str().unwrap_or("Unknown Country").to_string();
//                 Ok((ip, country))
//             } else {
//                 Err("Errore nel parsing della risposta.".to_string())
//             }
//         }
//         Err(e) => Err(e.to_string()),
//     }
// }