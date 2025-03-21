use eframe::egui;
use crate::chromium::{self, ChromiumProfile};
use crate::proxy_manager::ProxyConfig;
use crate::fingerprint_manager::SingleFingerprint;
use crate::ProfileManager;

use std::path::Path;

use rfd::FileDialog;

use serde::{Serialize, Deserialize};

use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Browsers {
    pub name: String,
    pub path: String,
}

pub fn save_custom_browsers(custom_browsers: &[Browsers]) {
    let json = serde_json::to_string_pretty(custom_browsers).expect("Failed to serialize custom browsers");
    let mut file = fs::File::create("custom_browsers.json").expect("Failed to create JSON file");
    file.write_all(json.as_bytes()).expect("Failed to write to JSON file");
}

pub fn load_custom_browsers() -> Vec<Browsers> {
    if let Ok(json) = fs::read_to_string("custom_browsers.json") {
        serde_json::from_str(&json).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    }
}

pub fn discover_installed_browsers() -> Vec<Browsers> {
    let mut browsers = Vec::new();

    // Checking common paths for installed browsers

    // Windows
    // Google Chrome
    if Path::new(r"C:\Program Files\Google\Chrome\Application\chrome.exe").exists() {
        browsers.push(Browsers {
            name: "Google Chrome".to_string(),
            path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
        });
    }

    // Brave
    // if Path::new(r"C:\Users\crystal\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe").exists() {
    //     browsers.push(Browsers {
    //         name: "Brave".to_string(),
    //         path: r"C:\Users\crystal\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe".to_string(),
    //     });
    // }

    // Edge
    if Path::new(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe").exists() {
        browsers.push(Browsers {
            name: "Edge".to_string(),
            path: r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string(),
        });
    }

    browsers
}

pub fn create_new_profile_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {

    // Button to add a custom browser
    ui.horizontal(|ui| {
        ui.label("Custom Browser Name:");
        ui.add(egui::TextEdit::singleline(&mut manager.custom_browser_name)); // Usa il campo di ProfileManager
    
        if ui.button("Add Custom Browser").clicked() {
            if let Some(file) = FileDialog::new().add_filter("Executable", &["exe"]).pick_file() {
                let custom_browser = Browsers {
                    name: if manager.custom_browser_name.trim().is_empty() {
                        "Custom Browser".to_string() // Nome predefinito se l'utente non inserisce nulla
                    } else {
                        manager.custom_browser_name.clone()
                    },
                    path: file.to_string_lossy().to_string(),
                };
                // Check if the browser is already in the list
                if !manager.installed_browsers.iter().any(|browser| browser.path == custom_browser.path) {
                    manager.installed_browsers.push(custom_browser.clone());
                    save_custom_browsers(&manager.installed_browsers);
                    manager.log_message = format!("Custom browser added: {}", custom_browser.name);
                } else {
                    manager.log_message = format!("Browser already exists: {}", custom_browser.path);
                }
                manager.custom_browser_name.clear(); // Pulisce il campo dopo l'aggiunta
            }
        }
    });

    ui.heading("NEW PROFILE");

    // Dropdown menu in order to select the browser
    let mut selected_browser_path = String::new();
    ui.horizontal(|ui| {
        ui.label("Select Browser:");
        egui::ComboBox::from_id_salt(egui::Id::new("browser_selector"))
            .selected_text(
                manager.installed_browsers
                    .iter()
                    .find(|browser| browser.path == manager.selected_browser_path)
                    .map(|browser| browser.name.clone())
                    .unwrap_or_else(|| "Select a browser".to_string()),
            )
            .show_ui(ui, |ui| {
                for browser in &manager.installed_browsers {
                    if ui
                        .selectable_value(
                            &mut selected_browser_path,
                            browser.path.clone(),
                            &browser.name,
                        )
                        .clicked()
                    {
                        manager.selected_browser_path = browser.path.clone();
                    }
                }
            });
    });

    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut manager.new_profile_name)); 

        if ui.button("CREATE").clicked() {
            if !manager.new_profile_name.trim().is_empty() {
                let new_profile = ChromiumProfile {
                    name: manager.new_profile_name.clone(),
                    browser_path: manager.selected_browser_path.clone(),
                    path: chromium::get_profile_dir(&manager.new_profile_name.clone()),
                    debugging_port: 0,
                    headless: true,
                    proxy: ProxyConfig::new(),
                    webrtc: String::new(),
                    custom_flags: String::new(),
                    fingerprint: SingleFingerprint {
                        os_type: String::new(),
                    },
                };
                manager.profiles.push(new_profile.clone());
                match chromium::create_new_profile(new_profile.clone()) {
                    Ok(_) => manager.log_message = format!("Profile {} created successfully.", new_profile.name),
                    Err(e) => manager.log_message = format!("Errore: {}", e),
                }
                chromium::save_profile_configs(&manager.profiles);
                manager.new_profile_name.clear();
                manager.log_message = format!("Profile {} created successfully.", new_profile.name);
            } else {
                manager.log_message = format!("Error: provide a valid name.");
            }
        }
    });
    ui.separator();
}