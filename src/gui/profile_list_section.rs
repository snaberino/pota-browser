use crate::ProfileManager;
use eframe::egui;

use crate::chromium::chromium::ChromiumProfile;

use crate::gui::render_util::{ render_webrtc_dropdown, render_debug_checkbox, render_headless_checkbox };

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
                ui.label("Debug Mode");
                // ui.label("Debugging Port");
                ui.label("WebRTC");
                ui.label("Actions");
                ui.end_row();

                // Table rows
                let mut profiles = manager.profiles.clone();
                for profile in profiles.iter_mut() {
                    ui.label(&profile.name);

                    // Checkbox per headless mode
                    render_headless_checkbox( ui, profile, &mut manager.profiles, &mut manager.log_message );

                    // Activate debugging port checkbox
                    render_debug_checkbox(ui, profile, &mut manager.profiles, &mut manager.log_message );

                    // WebRTC dropdown menu
                    render_webrtc_dropdown( ui, &mut profile.webrtc, &mut profile.name, &mut manager.profiles, &mut manager.log_message);

                    if ui.button("START").clicked() {
                        profile.headless = false;
                        match ChromiumProfile::open_chromium(&profile) {
                            Ok(_) => {
                                manager.log_message = format!("Profile {} opened successfully.", manager.selected_profile.name);
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