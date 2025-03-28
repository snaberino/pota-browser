use crate::ProfileManager;
use eframe::egui;
use crate::chromium::chromium::close_chrome;

pub fn active_profiles_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.heading("ACTIVE PROFILES");

    // Crea una tabella
    egui::Grid::new("active_profiles_table")
        .striped(true)
        .show(ui, |ui| {
            // Intestazioni della tabella
            ui.label("Profile Name");
            ui.label("Actions");
            ui.end_row();

            // Righe della tabella
            for profile in manager.open_profiles.clone() {
                ui.label(&profile.name);
                if ui.button("CLOSE").clicked() {
                    match close_chrome(&profile.name) {
                        Ok(_) => manager.log_message = format!("Profile {} closed successfully.", profile.name),
                        Err(e) => manager.log_message = format!("Error: {}", e),
                    }
                    manager.open_profiles.retain(|p| p.name != profile.name); // Rimuovi il profilo dalla lista
                }
                ui.end_row();
            }
        });

    ui.separator();
}