use crate::ProfileManager;
use eframe::egui;

pub fn profile_list_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.label("PROFILE LIST");
    egui::ScrollArea::vertical().show(ui, |ui| {
        for profile in &mut manager.profiles {
            ui.horizontal(|ui| {
                ui.label(&profile.name);
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
            }); // horizontal
        }
    });
    ui.separator();
}