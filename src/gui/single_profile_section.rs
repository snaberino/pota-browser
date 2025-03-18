use eframe::egui;
use crate::chromium::{open_chrome, save_profile_configs};
// use crate::proxy_manager::ProxyConfig;
use crate::ProfileManager;

pub fn single_profile_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.label("PROFILE CONFIGURATION");
    ui.horizontal(|ui| {
        ui.label("Select profile");
        egui::ComboBox::from_id_salt(egui::Id::new("profile_selector"))
        .selected_text(&manager.selected_profile.name)
        .show_ui(ui, |ui| {
            for profile in &manager.profiles {
                ui.selectable_value(
                    &mut manager.selected_profile,
                    profile.clone(),
                    &profile.name,
                );
            }
        });
        if manager.profiles.is_empty() {
            ui.add_enabled(false, egui::Button::new("OPEN"));
            manager.log_message = "No profile found.".to_string();
        } else {
            if ui.button("OPEN").clicked() {
                match open_chrome(manager.selected_profile.clone()) {
                    Ok(_) => {
                        manager.log_message = format!("Profile {} opened successfully.", manager.selected_profile.name);
                        manager.open_profiles.push(manager.selected_profile.clone());
                    }
                    Err(e) => manager.log_message = format!("Error: {}", e),
                };
            }
            
            // Debugging port checkbox, allow user to enable or disable debugging port for the selected profile
            let mut debug_enabled = manager.selected_profile.debugging_port != 0;
            if ui.checkbox(&mut debug_enabled, "Debug Mode").changed() {
                manager.selected_profile.debugging_port = if debug_enabled { 9222 } else { 0 };
                // Update the profile in the profiles list
                if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                    profile.debugging_port = manager.selected_profile.debugging_port;
                }
                save_profile_configs(&manager.profiles);
                manager.log_message = format!("Debugging port for profile {} set to {}.", manager.selected_profile.name, manager.selected_profile.debugging_port);
            }
            
            // Checkbox per headless mode, allow user to enable or disable headless mode for the selected profile
            let mut headless_enabled = manager.selected_profile.headless;
            if ui.checkbox(&mut headless_enabled, "Headless Mode").changed() {
                manager.selected_profile.headless = if headless_enabled { true } else { false };
                if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                    profile.headless = manager.selected_profile.headless;
                }
                save_profile_configs(&manager.profiles);
                manager.log_message = format!("Headless mode for profile {} set to {}.", manager.selected_profile.name, manager.selected_profile.headless);
            }

        }
    }); //  horizontal

    ui.horizontal(|ui| {
        ui.label("Proxy");
        egui::ComboBox::from_id_salt(egui::Id::new("proxy_selector"))
        .selected_text(&manager.selected_proxy.proxy_name)
        .show_ui(ui, |ui| {
            for proxy in &mut manager.proxy_configs {
                ui.selectable_value(&mut manager.selected_proxy, proxy.clone(), &proxy.proxy_name);
            }
        });
        if ui.button("Set Proxy").clicked() {
            manager.selected_profile.proxy = manager.selected_proxy.clone();
            manager.log_message = format!("Proxy {} set for profile {}.", manager.selected_proxy.proxy_name, manager.selected_profile.name);
            println!("Proxy selected: {:?}", manager.selected_proxy);
            println!("Profile selected: {:?}", manager.selected_profile);
        }
    }); // horizontal

    ui.label("FINGERPRINT CONFIGURATION");
    ui.horizontal(|ui|{
        ui.label("OS");
        egui::ComboBox::from_id_salt(egui::Id::new("os_selector"))
            .selected_text(&manager.single_fingerprint.os_type)
                .show_ui(ui, |ui| {
                    ui.horizontal(|ui| { 
                        if ui.button("Windows").clicked() {
                            manager.selected_os_list = manager.fingerprint_manager.os_type[0].clone();
                            println!("List A selected"); 
                        }
                        if ui.button("Mac").clicked() {
                            manager.selected_os_list = manager.fingerprint_manager.os_type[1].clone();
                            println!("List B selected");
                        }
                    });

                    for os_type in &manager.selected_os_list {
                        ui.selectable_value(&mut manager.single_fingerprint.os_type, os_type.clone(), os_type);
                    }

                });
    }); // horizontal

    // Dropdown menu for WebRTC IP handling
    ui.horizontal(|ui| {    
        // Store the old value
        let old_webrtc = manager.selected_profile.webrtc.clone();
        ui.label("WebRTC");
        egui::ComboBox::from_id_salt(egui::Id::new("webrtc_spoofing_selector"))
            .selected_text(&manager.selected_profile.webrtc)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut manager.selected_profile.webrtc, "default".to_string(), "default");
                ui.selectable_value(&mut manager.selected_profile.webrtc, "fake".to_string(), "fake");
                ui.selectable_value(&mut manager.selected_profile.webrtc, "block".to_string(), "block");
            });
        // Update only if changed
        if old_webrtc != manager.selected_profile.webrtc {
            if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                profile.webrtc = manager.selected_profile.webrtc.clone();
                save_profile_configs(&manager.profiles);
                manager.log_message = format!(
                    "WebRTC spoofing for profile {} set to {}.",
                    manager.selected_profile.name,
                    manager.selected_profile.webrtc
                );
            }
        }
    });

    // Custom FLAG
    ui.horizontal(|ui| {
        ui.label("Custom Flags");
        // Text box for custom flags
        ui.text_edit_singleline(&mut manager.selected_profile.custom_flags);

        if ui.button("Apply Flags").clicked() {
            if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                profile.custom_flags = manager.selected_profile.custom_flags.clone();
                save_profile_configs(&manager.profiles);
                manager.log_message = format!(
                    "Custom flags for profile {} set to: {}",
                    manager.selected_profile.name,
                    manager.selected_profile.custom_flags
                );
            }
        }
    });
    
    ui.separator();
}