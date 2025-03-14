use eframe::egui;
use crate::chrome::{self, ChromeProfile};
use crate::proxy_manager::ProxyConfig;
use crate::fingerprint_manager::SingleFingerprint;
use crate::ProfileManager;

pub fn create_new_profile_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.label("Create a new profile:");
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut manager.new_profile_name)); 

        if ui.button("CREATE").clicked() {
            if !manager.new_profile_name.trim().is_empty() {
                let new_profile = ChromeProfile {
                    name: manager.new_profile_name.clone(),
                    path: chrome::get_profile_dir(&manager.new_profile_name.clone()),
                    debugging_port: 0,
                    headless: true,
                    proxy: ProxyConfig::new(),
                    webrtc: String::new(),
                    fingerprint: SingleFingerprint {
                        os_type: String::new(),
                    },
                };
                manager.profiles.push(new_profile.clone());
                match chrome::create_new_profile(new_profile.clone()) {
                    Ok(_) => manager.log_message = format!("Profile {} created successfully.", new_profile.name),
                    Err(e) => manager.log_message = format!("Errore: {}", e),
                }
                chrome::save_profile_configs(&manager.profiles);
                manager.new_profile_name.clear();
                manager.log_message = format!("Profile {} created successfully.", new_profile.name);
            } else {
                manager.log_message = format!("Error: provide a valid name.");
            }
        }
    });
    ui.separator();
}