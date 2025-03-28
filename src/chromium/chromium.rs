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

// Get the profile directory
pub fn get_profile_dir(profile_name: &str) -> PathBuf {
    // I'm getting the current directory of the script
    let script_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(""));
    let mut profile_dir = script_dir.clone();
    profile_dir.push("chromium_profiles");
    // New profile directory
    profile_dir.push(profile_name);
    profile_dir
}

// Load Chrome profiles from a JSON file
pub fn load_profile_configs() -> ChromiumProfiles {
    let file_path = "chromium_profiles.json";

    // Check if the file exists
    if !PathBuf::from(file_path).exists() {
        let empty_profiles: ChromiumProfiles = vec![];
        save_profile_configs(&empty_profiles);
        return empty_profiles;
    }

    let file = File::open(file_path).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).expect("Unable to read file");

    // If the file is empty, return an empty vector
    if content.trim().is_empty() {
        let empty_profiles: ChromiumProfiles = vec![];
        save_profile_configs(&empty_profiles);
        empty_profiles
    } else {
        serde_json::from_str(&content).expect("Unable to parse JSON")
    }
}

// Save Chrome profiles to a JSON file
pub fn save_profile_configs(profiles_config: &ChromiumProfiles) {
    let file = File::create("chromium_profiles.json").expect("Unable to create file");
    serde_json::to_writer_pretty(file, profiles_config).expect("Unable to write JSON");
}

// Open a Chrome profile
pub fn open_chrome(profile: ChromiumProfile) -> io::Result<()> {
    println!("Opening a new Chrome profile in this directory: {}", profile.path.to_str().unwrap()); //debugging

    let chrome_path = profile.browser_path.clone();

    // Building the command to open Chrome with all the necessary arguments
    let mut command = Command::new(chrome_path);

    command.arg(format!("--user-data-dir={}", profile.path.to_str().unwrap()));
    command.arg("--no-first-run"); // Avoid first run dialog
    command.arg("--no-default-browser-check"); // Stop browser from asking to be the default browser
    command.arg("--hide-crash-restore-bubble"); // Disable session crashed pop up
    command.arg("--disable-features=Translate,LensStandalone,LensOverlay,LensOverlayTranslateButton,LensOverlayContextualSearchBox,LensOverlayLatencyOptimizations,LensOverlayImageContextMenuActions,LensOverlayTranslateLanguages,LensOverlaySidePanelOpenInNewTab"); // Stop browser from asking to translate pages and stop Google Lens

    // Add remote debugging if debugging_port is not 0
    if profile.debugging_port != 0 {
        command.arg(format!("--remote-debugging-port={}", profile.debugging_port));
    }
    // Headless mode
    if profile.headless == true {
        command.arg("--headless");
    }
    // Proxy server argument name if it's configured we use the proxy, need to refactor this because if name is empty but other proxy parameters are set it will not work
    // When proxy is active i need to set extra arguments to Chrome for --lang and --accept-language
    if profile.proxy.proxy_name != "" {
        command.arg(format!("--proxy-server={}://{}:{}",
            profile.proxy.proxy_type,
            profile.proxy.proxy_host,
            profile.proxy.proxy_port
        ));
        command.arg(format!("--lang={}", profile.proxy.lang_arg));
        command.arg(format!("--accept-lang={}", profile.proxy.accept_language_arg));
    }

    // WebRRTC Spoofing options

    if profile.webrtc == "block" {
        command.arg("--webrtc-ip-handling-policy=disable_non_proxied_udp");
        command.arg("--force-webrtc-ip-handling-policy");
    }

    // Custom flags option
    if profile.custom_flags != "" {
        command.arg(&profile.custom_flags);
    }

    // Spawn the process and store it in the CHROME_PROCESSES map in order to kill it later or other operations
    match command.spawn() {
        Ok(child) => {
            let mut processes = CHROMIUM_PROCESSES.lock().unwrap();
            processes.insert(profile.name.clone(), child);

            // Trying new way to connecto to Chrome DevTools Protocol

            if profile.debugging_port != 0 {
                websocket::cdp_main(profile.clone());
            }

        }
        Err(e) => {
            eprintln!("Error while opening Chrome: {}", e); //debugging
        }
    }
    println!("Chrome opened successfully!"); //debugging

    Ok(())
}

pub fn close_chrome(profile_name: &str) -> io::Result<()> {
    let mut processes = CHROMIUM_PROCESSES.lock().unwrap();
    if let Some(mut child) = processes.remove(profile_name) {
        if let Err(e) = child.kill() {
            eprintln!("Error while closing Chrome: {}", e);//debugging
        }
    } else {
        eprintln!("No path found for profile with name: {}", profile_name);//debugging
    }
    Ok(())
}

// When creating a new profile, actually i open it headless and then close it. So the whole folder structure is created.
pub fn create_new_profile(new_profile: ChromiumProfile) -> io::Result<()> {
    println!("New path directory:{}", new_profile.path.to_str().unwrap()); //debugging

    // NEED TO IMPLEMENT A WAY TO CHECK IF THE PROFILE ALREADY EXISTS

    match open_chrome(new_profile.clone()) {
        Ok(_) => {
            println!("Chrome opened successfully!");
        }
        Err(e) => {
            eprintln!("Error while opening Chrome: {}", e);
        }
    };

    match close_chrome(&new_profile.name) {
        Ok(_) => {
            println!("Chrome closed successfully!");
        }
        Err(e) => {
            eprintln!("Error while closing Chrome: {}", e);
        }

    };

    Ok(())
}
