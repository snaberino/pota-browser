// use reqwest::Client;
use reqwest::Client;

use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fs::File;
use std::io::{ Read, Write };
use std::io::BufReader;

use tokio::task;
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ProxyConfig {
    pub proxy_name: String,
    pub proxy_type: String,
    pub proxy_host: String,
    pub proxy_port: String,
    pub proxy_username: String,
    pub proxy_password: String,

    pub country: String,
    pub last_ip: String,
    pub used_ips: Vec<String>,
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
    println!("Saving proxy configs: {:?}", proxy_configs); // debugging
    let json = serde_json::to_string_pretty(proxy_configs).expect("Unable to serialize proxy configs");
    let mut file = File::create("proxy_config.json").expect("Unable to create file");
    file.write_all(json.as_bytes()).expect("Unable to write data");
}

// Function to test a proxy and grabbing info about it, actually saving only IPs as last used and overall list of used IPs
async fn check_proxy(mut proxy_config: ProxyConfig) -> Result<ProxyConfig, String> {
    let proxy_url = format!(
        "{}://{}:{}@{}:{}",
        proxy_config.proxy_type,
        proxy_config.proxy_username,
        proxy_config.proxy_password,
        proxy_config.proxy_host,
        proxy_config.proxy_port
    );
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(proxy_url).map_err(|e| e.to_string())?)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get("https://ipinfo.io/json").send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>().await {
                println!("{:?}", json); //debugging
                let ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
                proxy_config.last_ip = ip.clone();

                if !proxy_config.used_ips.contains(&ip) {
                    proxy_config.used_ips.push(ip.clone());
                }
                // proxy_config.used_ips.push(ip.clone());

                let country: String = json["country"].as_str().unwrap_or("Unknown Country").to_string();
                proxy_config.country = country.clone();
        
                Ok(proxy_config.clone())
            } else {
                Err("Error parsing the response.".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn start_check_proxy(proxy_config: ProxyConfig, proxies_config: ProxiesConfig) -> JoinHandle<Result<ProxyConfig, String>> {
    task::spawn(async move {
        match check_proxy(proxy_config).await {
            Ok(proxy_config) => {
                println!("Proxy checked successfully: {:?}", proxy_config); // debugging
                if let Some(existing_proxy) = proxies_config.iter().position(|p| p.proxy_name == proxy_config.proxy_name) {
                    let mut proxies_config = proxies_config.clone();
                    proxies_config[existing_proxy] = proxy_config.clone();
                    save_proxy_configs(&proxies_config);
                }
                Ok(proxy_config)
            }
            Err(e) => {
                eprintln!("Error while checking the proxy: {}", e);
                Err(e)
            }
        }
    })
}