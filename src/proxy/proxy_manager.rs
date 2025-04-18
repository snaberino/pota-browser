use std::fs::File;
use std::io::Write;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use reqwest::{ Client, Proxy };

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ProxyConfig {
    pub id: u32,
    pub name: String,
    pub protocol: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,

    pub country: String,
    pub ip: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ProxyManager {
    pub proxies: Vec<ProxyConfig>,
}

impl ProxyConfig {
    pub fn default() -> Self {
        ProxyConfig {
            id: 0,
            name: String::new(),
            protocol: String::new(),
            host: String::new(),
            port: String::new(),
            username: String::new(),
            password: String::new(),
            country: String::new(),
            ip: String::new(),
        }
    }

    pub fn new_proxy(proxies: Vec<ProxyConfig>, name: String, protocol: String, host: String, port: String, username: String, password: String) -> Self {
        ProxyConfig {
            id: Self::get_next_id(proxies),
            name,
            protocol,
            host,
            port,
            username,
            password,
            country: String::new(),
            ip: String::new(),
        }
    }

    fn get_next_id(proxies: Vec<ProxyConfig>) -> u32 {
        let mut max_id = 0;
        for proxy in proxies {
            if proxy.id > max_id {
                max_id = proxy.id;
            }
        }
        max_id + 1
    }
}

impl ProxyManager {
    pub fn default() -> Self {
        if let Ok(file) = File::open("proxies_config.json") {
            if let Ok(proxies) = serde_json::from_reader::<_, ProxyManager>(&file) {
                return proxies
            } else {
                // File doesn't exist, create it
                println!("Error loading proxies_config.json");

            }
        } else {
            let _ = File::create("proxies_config.json").expect("Failed to create proxies_config.json");
        }
        Self {
            proxies: Vec::new(),
        }
    }

    pub fn add_new_proxy(&mut self, name: String, protocol: String, host: String, port: String, username: String, password: String) {
        let new_proxy = ProxyConfig::new_proxy(self.proxies.clone(), name, protocol, host, port, username, password);
        self.proxies.push(new_proxy);
        let _ = self.save(); // Save the new proxy to the file
    }

    pub fn update(&mut self, index: u32, updated_proxy: ProxyConfig) {
        if let Some(existing_proxy) = self.proxies.iter_mut().find(|p| p.id == index) {
            *existing_proxy = updated_proxy;
            let _ = self.save(); // Save the changes to the file
        }
    }

    pub fn remove_proxy(&mut self, index: usize) {
        if index < self.proxies.len() {
            self.proxies.remove(index);
            let _ = self.save(); // Save the changes to the file
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create("proxies_config.json")?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    pub fn start_check_proxy(&self, index: u32, proxy_config: ProxyConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = self.clone();

        tokio::spawn(async move {
            match manager.check_proxy(proxy_config).await {
            Ok(proxy_config) => {
                println!("Proxy checked successfully:"); // debugging
                manager.update(index, proxy_config);

            },
            Err(e) => {
                eprintln!("Error while checking the proxy: {}", e); // debugging
            }
        }});
        Ok(())
    }

    pub async fn check_proxy(&self, mut proxy_config: ProxyConfig) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
        println!("Checking proxy index");

        let proxy_url = format!(
            "{}://{}:{}@{}:{}",
            proxy_config.protocol,
            proxy_config.username,
            proxy_config.password,
            proxy_config.host,
            proxy_config.port,
        );
        println!("{}", proxy_url);
        let client = Client::builder()
            .proxy(Proxy::all(proxy_url).map_err(|e| e.to_string())?)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        match client.get("https://ipinfo.io/json").send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    // Saving the last IP
                    // println!("{:?}", json); // debugging
                    let ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
                    proxy_config.ip = ip.clone();

                    // Saving the country
                    let country: String = json["country"].as_str().unwrap_or("Unknown Country").to_string();
                    proxy_config.country = country.clone();

                    Ok(proxy_config)
                } else {
                    Err("Error parsing the response.".into())
                }
            }
            Err(e) => Err(e.to_string().into()),
        }
    }
}