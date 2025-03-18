use eframe::egui;
use crate::chromium::{self, ChromeProfile};
use crate::proxy_manager::ProxyConfig;
use crate::fingerprint_manager::SingleFingerprint;
use crate::ProfileManager;

use std::collections::HashMap;
use std::path::Path;

pub fn discover_installed_browsers() -> HashMap<String, String> {
    let mut browsers = HashMap::new();

    // Checking common paths for installed browsers

    // Windows
    // Google Chrome
    if Path::new(r"C:\Program Files\Google\Chrome\Application\chrome.exe").exists() {
        browsers.insert(
            "Google Chrome".to_string(),
            r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
        );
    }

    // Brave
    if Path::new(r"C:\Users\crystal\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe").exists() {
        browsers.insert(
            "Brave".to_string(),
            r"C:\Users\crystal\AppData\Local\BraveSoftware\Brave-Browser\Application\brave.exe".to_string(),
        );
    }

    // Edge
    if Path::new(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe").exists() {
        browsers.insert(
            "Edge".to_string(),
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string(),
        );
    }

    browsers
}

pub fn create_new_profile_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.label("NEW PROFILE");

    // Discovering installed browsers
    let mut selected_browser_path = String::new();

    // Dropdown to select the browser

    ui.horizontal(|ui| {
        ui.label("Select Browser:");
        egui::ComboBox::from_id_salt(egui::Id::new("browser_selector"))
            .selected_text(
                manager.installed_browsers
                    .iter()
                    .find(|(_, path)| path == &&manager.selected_browser_path)
                    .map(|(name, _)| name.clone())
                    .unwrap_or_else(|| "Select a browser".to_string()),
            )
            .show_ui(ui, |ui| {
                for (browser_name, browser_path) in &manager.installed_browsers {
                    if ui
                        .selectable_value(
                            &mut selected_browser_path,
                            browser_path.clone(),
                            browser_name,
                        )
                        .clicked()
                    {
                        manager.selected_browser_path = browser_path.clone();
                    }
                }
            });
    });

    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut manager.new_profile_name)); 

        if ui.button("CREATE").clicked() {
            if !manager.new_profile_name.trim().is_empty() {
                let new_profile = ChromeProfile {
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