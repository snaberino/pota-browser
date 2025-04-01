use crate::ProfileManager;
use eframe::egui;
use crate::chromium::chromium::ChromiumProfile;

pub fn active_profiles_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.heading("ACTIVE PROFILES");

    // Active profiles table
    egui::Grid::new("active_profiles_table")
        .striped(true)
        .show(ui, |ui| {
            // Table header
            ui.label("Profile Name");
            ui.label("Actions");
            ui.end_row();

            // Table rows
            for profile in manager.open_profiles.clone() {
                ui.label(&profile.name);
                if ui.button("CLOSE").clicked() {
                    match ChromiumProfile::close_chromium(&profile) {
                        Ok(_) => manager.log_message = format!("Profile {} closed successfully.", profile.name),
                        Err(e) => manager.log_message = format!("Error: {}", e),
                    }
                    manager.open_profiles.retain(|p| p.name != profile.name); // Remove the closed profile from the list
                }
                ui.end_row();
            }
        });

    ui.separator();
}