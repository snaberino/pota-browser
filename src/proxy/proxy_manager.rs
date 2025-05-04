use std::fs::File;
use std::io::Write;
use std::time::Duration;
use serde::{Deserialize, Serialize};
// use reqwest::{ Client, Proxy };
// use std::collections::HashMap;

#[derive(Clone)]
struct LanguageSettings {
    lang: &'static str,
    accept_language: &'static str,
}

// Define a map to store the language settings for each country
fn get_language_settings(country: &str) -> Option<LanguageSettings> {
    match country {
        "US" => Some(LanguageSettings { lang: "en-US", accept_language: "en-US,en;q=0.9" }),
        "IT" => Some(LanguageSettings { lang: "it-IT", accept_language: "it-IT,it;q=0.9" }),
        "FR" => Some(LanguageSettings { lang: "fr-FR", accept_language: "fr-FR,fr;q=0.9" }),
        // Add more countries as needed
        _ => None,
    }
}

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

    pub lang: String,
    pub accept_lang: String
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
            lang: String::new(),
            accept_lang: String::new(),
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
            lang: String::new(),
            accept_lang: String::new(),
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

    // pub async fn check_proxy(&self, mut proxy_config: ProxyConfig) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
    //     println!("Checking proxy index");

    //     let proxy_url = format!(
    //         "{}://{}:{}@{}:{}",
    //         proxy_config.protocol,
    //         proxy_config.username,
    //         proxy_config.password,
    //         proxy_config.host,
    //         proxy_config.port,
    //     );
    //     println!("{}", proxy_url);
    //     let client = Client::builder()
    //         .proxy(Proxy::all(proxy_url).map_err(|e| e.to_string())?)
    //         .timeout(Duration::from_secs(10))
    //         .build()
    //         .map_err(|e| e.to_string())?;

    //     match client.get("https://ipinfo.io/json").send().await {
    //         Ok(response) => {
    //             if let Ok(json) = response.json::<serde_json::Value>().await {
    //                 // Saving the last IP
    //                 let ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
    //                 proxy_config.ip = ip.clone();

    //                 // Saving the country
    //                 let country: String = json["country"].as_str().unwrap_or("Unknown Country").to_string();
    //                 proxy_config.country = country.clone();

    //                 // Set language and accept_language based on country
    //                 if let Some(lang_settings) = get_language_settings(&proxy_config.country) {
    //                     proxy_config.lang = lang_settings.lang.to_string();
    //                     proxy_config.accept_lang = lang_settings.accept_language.to_string();
    //                 } else {
    //                     println!("Warning: No language settings found for country {}", proxy_config.country);
    //                 }

    //                 Ok(proxy_config)
    //             } else {
    //                 Err("Error parsing the response.".into())
    //             }
    //         }
    //         Err(e) => Err(e.to_string().into()),
    //     }
    // }


    pub async fn check_proxy(&self, mut proxy_config: ProxyConfig) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
        println!("Checking proxy: {}", proxy_config.name);
    
        let client = if proxy_config.protocol == "http" {
            // HTTPS proxy via reqwest + CONNECT
            let proxy_url = format!(
                "{}://{}:{}@{}:{}",
                proxy_config.protocol,
                proxy_config.username,
                proxy_config.password,
                proxy_config.host,
                proxy_config.port,
            );
    
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url)?)
                .timeout(Duration::from_secs(10))
                .build()?
        } else if proxy_config.protocol == "socks5" {
            // SOCKS5 proxy (reqwest non supporta SOCKS5 + auth nativamente)
            // workaround: usare tokio-socks per stabilire connessione e leggere a mano
    
            use tokio_socks::tcp::Socks5Stream;
            // use tokio::net::TcpStream;
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
            let host = "ipinfo.io";
            let port = 443;
    
            let socks_stream = Socks5Stream::connect_with_password(
                (
                    proxy_config.host.as_str(),
                    proxy_config.port.parse::<u16>().unwrap_or(1080),
                ),
                (host, port),
                &proxy_config.username,
                &proxy_config.password,
            )
            .await?;
    
            let mut stream = tokio_native_tls::TlsConnector::from(
                native_tls::TlsConnector::new().unwrap()
            )
            .connect(host, socks_stream.into_inner())
            .await?;
    
            // Minimal GET request
            let request = b"GET /json HTTP/1.1\r\nHost: ipinfo.io\r\nConnection: close\r\n\r\n";
            stream.write_all(request).await?;
    
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await?;
    
            let response_str = String::from_utf8_lossy(&buf);
            let json_start = response_str.find("{").unwrap_or(0);
            let json_part = &response_str[json_start..];
    
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_part) {
                proxy_config.ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
                proxy_config.country = json["country"].as_str().unwrap_or("Unknown Country").to_string();
    
                if let Some(lang_settings) = get_language_settings(&proxy_config.country) {
                    proxy_config.lang = lang_settings.lang.to_string();
                    proxy_config.accept_lang = lang_settings.accept_language.to_string();
                }
    
                return Ok(proxy_config);
            } else {
                return Err("Failed to parse JSON from SOCKS5 response".into());
            }
        } else {
            return Err(format!("Unsupported proxy protocol: {}", proxy_config.protocol).into());
        };
    
        // Common HTTP flow (for HTTPS only)
        let res = client.get("https://ipinfo.io/json").send().await?;
        let json = res.json::<serde_json::Value>().await?;
    
        proxy_config.ip = json["ip"].as_str().unwrap_or("Unknown IP").to_string();
        proxy_config.country = json["country"].as_str().unwrap_or("Unknown Country").to_string();
    
        if let Some(lang_settings) = get_language_settings(&proxy_config.country) {
            proxy_config.lang = lang_settings.lang.to_string();
            proxy_config.accept_lang = lang_settings.accept_language.to_string();
        }
    
        Ok(proxy_config)
    }
    
}