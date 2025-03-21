use crate::ProfileManager;
use eframe::egui;
// use std::fs;

// use crate::chromium::save_profile_configs;

pub fn profile_list_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.heading("PROFILE LIST");

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("profile_table")
            .striped(true)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // Table header
                ui.label("Name");
                ui.label("Headless");
                ui.label("Debugging Port");

                ui.label("Actions");
                ui.end_row();

                // Table rows
                for profile in &mut manager.profiles {
                    ui.label(&profile.name);

                    // Headless column
                    ui.label(if profile.headless { "Yes" } else { "No" });

                    // Headless column
                    ui.label(profile.debugging_port.to_string());

                    if ui.button("START").clicked() {
                        profile.headless = false;
                        match crate::chromium::open_chrome(profile.clone()) {
                            Ok(_) => {
                                manager.log_message = format!("Profile {} opened successfully.", profile.name);
                                manager.open_profiles.push(profile.clone());
                            }
                            Err(e) => manager.log_message = format!("Error: {}", e),
                        };
                    }
                    ui.end_row();
                }
            });
    });

    ui.separator();
}