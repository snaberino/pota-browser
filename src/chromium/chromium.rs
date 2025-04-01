use std::io::{self, Read, BufReader};
use std::fs::File;
use std::path::PathBuf;
use std::process::{ Command, Child };
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};

use lazy_static::lazy_static;

use crate::proxy_manager::ProxyConfig;
use crate::chromium::websocket;
use crate::fingerprint_manager::SingleFingerprint;

lazy_static! {
    static ref CHROMIUM_PROCESSES: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Chrome profile structure
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ChromiumProfile {
    pub name: String, // Profile name
    pub browser_path: String, // Browser path
    pub path: PathBuf, // Profile path folder
    pub debugging_port : u16, // Debugging port
    pub headless: bool, // Headless mode
    pub proxy: ProxyConfig, // Proxy configuration
    pub webrtc: String, // WebRTC Spoofing
    pub custom_flags: String, // Custom flags
    pub images: u8, // 0 = no images, 1 = load images, 2 = load only captcha images
    pub fingerprint: SingleFingerprint,
}

pub type ChromiumProfiles = Vec<ChromiumProfile>;

impl ChromiumProfile {

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            browser_path: "browser_path".to_string(),
            path: Self::get_profile_dir(name),
            debugging_port: 0,
            headless: false,
            proxy: ProxyConfig::new(),
            webrtc: String::new(),
            custom_flags: String::new(),
            images: 1,
            fingerprint: SingleFingerprint {
                os_type: String::new(),
            },
        }
    }

    pub fn open_chromium(&self) -> io::Result<()> {
        let mut command = Command::new(&self.browser_path);
        
        self.add_extra_args(&mut command);
        self.add_debugging_port(&mut command);
        self.add_headless_flag(&mut command);
        self.add_proxy_settings(&mut command);
        self.add_webrtc_settings(&mut command);
        self.add_custom_flags(&mut command);

        // command.arg(format!("--proxy-server={}", "socks5://127.0.0.1:3000"));
        // command.arg("--host-resolver-rules=\"MAP * ~NOTFOUND , EXCLUDE 127.0.0.1\"");


    // Spawn the process and store it in the CHROMIUM_PROCESSES map in order to kill it later or other operations
        match command.spawn() {
            Ok(child) => {
                let mut processes = CHROMIUM_PROCESSES.lock().unwrap();
                processes.insert(self.name.clone(), child);

                // MIT for Chrome DevTools Protocol

                if self.debugging_port != 0 {
                    websocket::cdp_main(self.clone());
                }

            }
            Err(e) => {
                eprintln!("Error while opening Chrome: {}", e); //debugging
            }
        }
        println!("Chrome opened successfully!"); //debugging
        Ok(())
    }

    pub fn close_chromium(&self) -> io::Result<()> {
        let mut processes = CHROMIUM_PROCESSES.lock().unwrap();
        if let Some(mut child) = processes.remove(&self.name) {
            if let Err(e) = child.kill() {
                eprintln!("Error while closing Chrome: {}", e);//debugging
            }
        } else {
            eprintln!("No path found for profile with name: {}", self.name);//debugging
        }
        Ok(())
    }

    pub fn create_chromium(&self) -> io::Result<()> {
        println!("Creating new profile: {}", self.name);

        self.open_chromium()?;
        self.close_chromium()?;

        Ok(())
    }

    pub fn load_profile_configs() -> ChromiumProfiles {
        let file_path = "chromium_profiles.json";

        // Check if the file exists
        if !PathBuf::from(file_path).exists() {
            let empty_profiles: ChromiumProfiles = vec![];
            ChromiumProfile::save_profile_configs(&empty_profiles);
            return empty_profiles;
        }

        let file = File::open(file_path).expect("Unable to open file");
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).expect("Unable to read file");

        // If the file is empty, return an empty vector
        if content.trim().is_empty() {
            let empty_profiles: ChromiumProfiles = vec![];
            ChromiumProfile::save_profile_configs(&empty_profiles);
            empty_profiles
        } else {
            serde_json::from_str(&content).expect("Unable to parse JSON")
        }
    }

    pub fn save_profile_configs(profiles_config: &ChromiumProfiles) {
        let file = File::create("chromium_profiles.json").expect("Unable to create file");
        serde_json::to_writer_pretty(file, profiles_config).expect("Unable to write JSON");
    }

    pub fn get_profile_dir(profile_name: &str) -> PathBuf {
        // I'm getting the current directory of the script
        let script_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(""));
        let mut profile_dir = script_dir.clone();
        profile_dir.push("chromium_profiles");
        // New profile directory
        profile_dir.push(profile_name);
        profile_dir
    }

    // Helper functions

    fn add_extra_args(&self, command: &mut Command) {
        command.arg(format!("--user-data-dir={}", self.path.to_str().unwrap()));
        command.arg("--no-first-run");
        command.arg("--no-default-browser-check");
        command.arg("--hide-crash-restore-bubble");
        command.arg("--disable-features=Translate,LensStandalone,LensOverlay,LensOverlayTranslateButton,LensOverlayContextualSearchBox,LensOverlayLatencyOptimizations,LensOverlayImageContextMenuActions,LensOverlayTranslateLanguages,LensOverlaySidePanelOpenInNewTab");
    }

    fn add_debugging_port(&self, command: &mut Command) {
        if self.debugging_port != 0 {
            command.arg(format!("--remote-debugging-port={}", self.debugging_port));
        }
    }

    fn add_headless_flag(&self, command: &mut Command) {
        if self.headless {
            command.arg("--headless");
        }
    }

    fn add_proxy_settings(&self, command: &mut Command) {
        if !self.proxy.proxy_name.is_empty() {
            command.arg(format!(
                "--proxy-server={}://{}:{}",
                self.proxy.proxy_type, self.proxy.proxy_host, self.proxy.proxy_port
            ));
            command.arg(format!("--lang={}", self.proxy.lang_arg));
            command.arg(format!("--accept-lang={}", self.proxy.accept_language_arg));
        }
    }

    fn add_webrtc_settings(&self, command: &mut Command) {
        if self.webrtc == "block" {
            command.arg("--webrtc-ip-handling-policy=disable_non_proxied_udp");
            command.arg("--force-webrtc-ip-handling-policy");
        }
    }

    fn add_custom_flags(&self, command: &mut Command) {
        if !self.custom_flags.is_empty() {
            command.arg(&self.custom_flags);
        }
    }
}