use crate::ProfileManager;
use eframe::egui;

pub fn active_profiles_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.label("Active Profile:");
    for profile in manager.open_profiles.clone() {
        ui.horizontal(|ui| {
            ui.label(&profile.name);
            if ui.button("CLOSE").clicked() {
                match crate::chrome::close_chrome(&profile.name) {
                    Ok(_) => manager.log_message = format!("Profile {} closed successfully.", profile.name),
                    Err(e) => manager.log_message = format!("Error: {}", e),
                }
                manager.open_profiles.retain(|p| p.name != profile.name); // Rimuovi il profilo dalla lista
            }
        });
    }
    ui.separator();
}