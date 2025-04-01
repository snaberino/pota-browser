use eframe::egui;
use crate::chromium::chromium::ChromiumProfile;

use crate::gui::render_util::{ render_webrtc_dropdown, render_headless_checkbox, render_debug_checkbox };

use crate::ProfileManager;

pub fn single_profile_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.heading("PROFILE CONFIGURATION");
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

                match ChromiumProfile::open_chromium(&manager.selected_profile) {
                    Ok(_) => {
                        manager.log_message = format!("Profile {} opened successfully.", manager.selected_profile.name);
                        manager.open_profiles.push(manager.selected_profile.clone());
                    }
                    Err(e) => manager.log_message = format!("Error: {}", e),
                };
            }
        }
    }); //  horizontal
    ui.horizontal(|ui| {
        ui.label("Headless mode:");
        render_headless_checkbox( ui, &mut manager.selected_profile, &mut manager.profiles, &mut manager.log_message );
    });

    ui.horizontal(|ui| {
        ui.label("Debugging mode:");
        render_debug_checkbox(ui, &mut manager.selected_profile, &mut manager.profiles, &mut manager.log_message);
    });

    // Images configuration
    ui.horizontal(|ui| {
        ui.label("Images:");
        egui::ComboBox::from_id_salt("Block Images")
            .selected_text(match manager.selected_profile.images {
                0 => "Block all",
                1 => "Allow all",
                2 => "Allow only from CAPTCHA Providers",
                _ => "Unknown",
            })
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut manager.selected_profile.images, 0, "Block all").clicked() {
                    if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                        profile.images = 0;
                        ChromiumProfile::save_profile_configs(&manager.profiles);
                        manager.log_message = format!("Images for profile {} set to allow all.", manager.selected_profile.name);
                    }
                };
                if ui.selectable_value(&mut manager.selected_profile.images, 1, "Allow all").clicked() {
                    if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                        profile.images = 1;
                        ChromiumProfile::save_profile_configs(&manager.profiles);
                        manager.log_message = format!("Images for profile {} set to block all.", manager.selected_profile.name);
                    }
                };
                if ui.selectable_value(&mut manager.selected_profile.images, 2, "Allow only from CAPTCHA Providers").clicked() {
                    if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                        profile.images = 2;
                        ChromiumProfile::save_profile_configs(&manager.profiles);
                        manager.log_message = format!("Images for profile {} set to allow from CAPTCHA Providers.", manager.selected_profile.name);
                    }
                }
            });
    }); // horizontal

    ui.horizontal(|ui| {
        ui.label("Proxy");
        egui::ComboBox::from_id_salt(egui::Id::new("proxy_selector"))
        .selected_text(&manager.selected_proxy.proxy_name)
        .show_ui(ui, |ui| {
            for proxy in &mut manager.proxy_configs {
                ui.selectable_value(&mut manager.selected_proxy, proxy.clone(), &proxy.proxy_name);
            }
        });
        if ui.button("SET PROXY").clicked() {
            manager.selected_profile.proxy = manager.selected_proxy.clone();
            manager.log_message = format!("Proxy {} set for profile {}.", manager.selected_proxy.proxy_name, manager.selected_profile.name);
            println!("Proxy selected: {:?}", manager.selected_proxy);
            println!("Profile selected: {:?}", manager.selected_profile);
        }
    }); // horizontal

    ui.heading("FINGERPRINT CONFIGURATION");
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

        render_webrtc_dropdown(
            ui,
            &mut manager.selected_profile.webrtc,
            &manager.selected_profile.name,
            &mut manager.profiles,
            &mut manager.log_message,
        );
    });

    // Custom FLAG
    ui.horizontal(|ui| {
        ui.label("Custom Flags");
        // Text box for custom flags
        ui.text_edit_singleline(&mut manager.selected_profile.custom_flags);

        if ui.button("APPLY FLAGS").clicked() {
            if let Some(profile) = manager.profiles.iter_mut().find(|p| p.name == manager.selected_profile.name) {
                profile.custom_flags = manager.selected_profile.custom_flags.clone();
                ChromiumProfile::save_profile_configs(&manager.profiles);
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