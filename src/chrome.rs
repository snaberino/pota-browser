use std::io::{self, Read};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{ Command, Child };
use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use crate::proxy_manager::ProxyConfig;
use crate::websocket;

lazy_static! {
    static ref CHROME_PROCESSES: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Chrome profile structure
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ChromeProfile {
    pub name: String,
    pub path: PathBuf,
    pub debugging_port : u16,
    pub headless: bool,
    pub proxy: ProxyConfig,
}

pub type ChromeProfiles = Vec<ChromeProfile>;

// Get the profile directory
pub fn get_profile_dir(profile_name: &str) -> PathBuf {
    // I'm getting the current directory of the script
    let script_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(""));
    let mut profile_dir = script_dir.clone();
    profile_dir.push("chrome_profiles");
    // New profile directory
    profile_dir.push(profile_name);
    profile_dir
}

// Load Chrome profiles from a JSON file
pub fn load_profile_configs() -> ChromeProfiles {
    let file_path = "chrome_profiles.json";
    let file = File::open(file_path).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).expect("Unable to read file");

    // If the file is empty, return an empty vector
    if content.trim().is_empty() {
        let empty_profiles: ChromeProfiles = vec![];
        save_profile_configs(&empty_profiles);
        empty_profiles
    } else {
        serde_json::from_str(&content).expect("Unable to parse JSON")
    }
}

// Save Chrome profiles to a JSON file
pub fn save_profile_configs(profiles_config: &ChromeProfiles) {
    let file = File::create("chrome_profiles.json").expect("Unable to create file");
    serde_json::to_writer_pretty(file, profiles_config).expect("Unable to write JSON");
}

// Open a Chrome profile
pub fn open_chrome(profile: ChromeProfile) -> io::Result<()> {
    println!("Opening a new Chrome profile in this directory: {}", profile.path.to_str().unwrap()); //debugging

    // Full path to the Chrome executable, to enhance compatibility for other OSs
    let chrome_path = r"C:\Program Files\Google\Chrome\Application\chrome.exe";

    let mut command = Command::new(chrome_path);
    command.arg(format!("--user-data-dir={}", profile.path.to_str().unwrap()));
    command.arg("--no-first-run");
    // Add remote debugging if debugging_port is not 0
    if profile.debugging_port != 0 {
        command.arg(format!("--remote-debugging-port={}", profile.debugging_port));
    }
    // Headless mode
    if profile.headless == true {
        command.arg("--headless");
    }
    // Proxy server argument
    if profile.proxy.proxy_name != "" {
        command.arg(format!("--proxy-server={}://{}:{}",
            profile.proxy.proxy_type,
            profile.proxy.proxy_host,
            profile.proxy.proxy_port
        ));
    }

    // EXPERIMENTAL
    // Add experimental options for WebRTC
    
    // command.arg("--webrtc-stun-server='stun:localhost:3478'");
    command.arg("--force-webrtc-ip-handling-policy");
    // command.arg("--enforce-webrtc-ip-permission-check");
    // command.arg(format!("--disable-features=NetworkService,NetworkServiceInProcess"));

    // END EXPERIMENTAL
    
    // Spawn the process and store it in the CHROME_PROCESSES map in order to kill it later or other operations
    match command.spawn() {
        Ok(child) => {
            let mut processes = CHROME_PROCESSES.lock().unwrap();
            processes.insert(profile.name.clone(), child);
            if profile.proxy.proxy_name != "" {
                println!("Setting proxy...");
                // If the process spawn correctly, set the proxy
                match websocket::set_proxy_cdp(&profile) {
                    Ok(_) => {
                        println!("Proxy set successfully!");
                    }
                    Err(e) => {
                        eprintln!("Error while setting the proxy: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error while opening Chrome: {}", e);
        }
    }
    println!("Chrome opened successfully!");

    //EXPERIMENTAL

    // match websocket::set_timezone_cdp(&profile){
    //     Ok(_) => {
    //         println!("Timezone set successfully!");
    //     }
    //     Err(e) => {
    //         eprintln!("Error while setting the timezone: {}", e);
    //     }
    // }

    // END EXPERIMENTAL

    Ok(())
}

pub fn close_chrome(profile_name: &str) -> io::Result<()> {
    let mut processes = CHROME_PROCESSES.lock().unwrap();
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
pub fn create_new_profile(new_profile: ChromeProfile) -> io::Result<()> {
    println!("New path directory:{}", new_profile.path.to_str().unwrap()); //debugging

    // This don't work anymore
    // if new_profile.path.exists() {
    //     return Err(io::Error::new(
    //         io::ErrorKind::AlreadyExists,
    //         format!("Il profilo '{}' esiste già.", new_profile.name),
    //     ));
    // }

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
