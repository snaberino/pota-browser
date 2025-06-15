use std::fs::File;
use serde::{Deserialize, Serialize};
use std::io::Write;
use::std::process::{Command, Child};
use::std::collections::HashMap;
use::std::sync::{ Arc, Mutex, OnceLock };

use crate::proxy::proxy_server::ProxyServer;
use crate::proxy::proxy_manager::ProxyConfig;

static CHROMIUM_PROCESSES: OnceLock<Arc<Mutex<HashMap<String, Child>>>> = OnceLock::new();

fn get_chromium_processes() -> &'static Arc<Mutex<HashMap<String, Child>>> {
    CHROMIUM_PROCESSES.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChromiumProfile {
    pub id: u32,
    pub name: String,
    pub profile_path: String,
    pub browser_path: String,
    pub debugging_port: u32,
    pub headless: bool,
    pub webrtc: String,
    pub proxy: ProxyConfig,
    pub proxy_server_port: u16,
    pub is_running: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ChromiumManager {
    pub profiles: Vec<ChromiumProfile>,
}

impl ChromiumProfile {
    /// Creates a new, empty ChromiumProfile.
    pub fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            profile_path: String::new(),
            browser_path: String::new(),
            debugging_port: 0,
            headless: false,
            webrtc: "default".to_string(),
            proxy: ProxyConfig::default(),
            proxy_server_port: 0,
            is_running: false,
        }
    }

    /// Creates a new ChromiumProfile with the specified parameters.
    pub fn new_profile(profiles: Vec<ChromiumProfile>, name: String, profile_path: String, browser_path: String) -> Self {
        Self {
            id: Self::get_next_id(profiles),
            name: name.clone(),
            profile_path: if profile_path.is_empty() {
                Self::default_profile_path(name.clone())
            } else {
                profile_path
            },
            browser_path,
            debugging_port: 0,
            headless: false,
            webrtc: "default".to_string(),
            proxy: ProxyConfig::default(),
            proxy_server_port: 0,
            is_running: false,
        }
    }

    /// Determines the next available ID for a new profile.
    fn get_next_id(profiles: Vec<ChromiumProfile>) -> u32 {
        let mut max_id = 0;
        for profile in profiles {
            if profile.id > max_id {
                max_id = profile.id;
            }
        }
        max_id + 1
    }

    /// Generates the default profile path based on the profile name.
    fn default_profile_path(profile_name: String) -> String {
        let potabrowser_dir = std::env::current_dir().unwrap();
        let mut profile_dir = potabrowser_dir.clone();
        profile_dir.push("chromium_profiles");
        profile_dir.push(profile_name);
        profile_dir.to_str().unwrap().to_string()
    }

}

impl ChromiumManager {
    pub fn default() -> Self {
        if let Ok(file) = File::open("chromium_profiles.json") {
            if let Ok(profiles) = serde_json::from_reader::<_, ChromiumManager>(&file) {
                return profiles
            } else {
                println!("Error loading chromium_profiles.json");
            }
        } else {
            let _ = File::create("chromium_profiles.json").expect("Failed to create chromium_profiles.json");
        }
        Self {
            profiles: Vec::new(),
        }
    }

    pub fn add_new_profile(&mut self, name: String, profile_path: String, browser_path: String) {
        let new_profile = ChromiumProfile::new_profile(self.profiles.clone(), name.clone(), profile_path, browser_path);
        self.profiles.push(new_profile);
        let _ = self.save();
    }

    pub fn update(&mut self, index: u32, updated_profile: ChromiumProfile) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(profile) = self.profiles.iter_mut().find(|p| p.id == index) {
            *profile = updated_profile;
            self.save()?;
            Ok(())
        } else {
            Err("Profile not found".into())
        }
    }

    pub fn get_profiles(&self) -> Vec<ChromiumProfile> {
        self.profiles.clone()
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.profiles.len() {
            self.profiles.remove(index);
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create("chromium_profiles.json")?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    pub fn start(&mut self, mut profile: ChromiumProfile) -> Result<(), Box<dyn std::error::Error>> {
        if profile.proxy.name != "" {
            let proxy_server = ProxyServer::new();

            profile.proxy_server_port = self.get_available_port();

            let _ = proxy_server.start_server_proxy(&profile);   
        }
     
        let child = self.chromium_command_line(&profile).spawn()?;

        let mut processes = get_chromium_processes().lock().unwrap();
        processes.insert(profile.name.clone(), child);

        Ok(())
    }

    fn get_available_port(&mut self) -> u16 {
        let udp_socket = std::net::UdpSocket::bind(("127.0.0.1", 0)).unwrap();
        udp_socket.local_addr().unwrap().port()
    }

    fn chromium_command_line(&mut self, profile: &ChromiumProfile) -> Command {

        let mut command: Command = Command::new(&profile.browser_path);

        command.arg(format!("--user-data-dir={}", profile.profile_path));
        // command.arg(format!("--remote-debugging-port={}", profile.debugging_port));
        
        command.arg("--no-first-run");
        command.arg("--no-default-browser-check");
        command.arg("--hide-crash-restore-bubble");
        command.arg("--disable-features=Translate,LensStandalone,LensOverlay,LensOverlayTranslateButton,LensOverlayContextualSearchBox,LensOverlayLatencyOptimizations,LensOverlayImageContextMenuActions,LensOverlayTranslateLanguages,LensOverlaySidePanelOpenInNewTab");

        command.arg(format!("--lang={}", profile.proxy.lang));
        command.arg(format!("--accept-lang={}", profile.proxy.accept_lang));

        // Proxy config
        if profile.proxy.name != "" {
            command.arg(format!("--proxy-server=http://127.0.0.1:{}", profile.proxy_server_port));
        }
        // Headless config
        if profile.headless {
            command.arg("--headless");
        }
        // WebRTC config
        if profile.webrtc == "disabled" {
            command.arg("--webrtc-ip-handling-policy=disable_non_proxied_udp");
            command.arg("--force-webrtc-ip-handling-policy");
        }

        command
    }

    pub fn stop(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = get_chromium_processes().lock().unwrap().remove(name) {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn set_proxy(&mut self, profile_id: u32, proxy: ProxyConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(profile) = self.profiles.iter_mut().find(|p| p.id == profile_id) {
            profile.proxy = proxy;
            self.save()?;
            Ok(())
        } else {
            Err("Profile not found".into())
        }
    }
}
