use eframe::egui;
use crate::gui::custom_browser::{CustomBrowser, CustomBrowserPanel};
use crate::gui::logger::LoggerPanel;

use crate::chromium::chromium_manager::{ChromiumManager};

pub struct NewProfilePanel {
    selected_browser: Option<CustomBrowser>,
    new_profile_name: String,

    custom_profile_path_status: bool,
    custom_profile_path: String,
}

impl NewProfilePanel {
    pub fn new() -> Self {
        Self {
            selected_browser: None,
            new_profile_name: String::new(),

            custom_profile_path_status: false,
            custom_profile_path: String::new(),

        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, logger: &mut LoggerPanel, custom_browsers: &mut CustomBrowserPanel, chromium_manager: &mut ChromiumManager) {
        ui.heading("New profile");
        egui::Grid::new("new_profiles_grid").show(ui, |ui| {
            ui.label("Select Browser");
            ui.label("Custom Path");
            ui.label("Path");
            ui.label("Profile Name");
            ui.end_row();
            egui::ComboBox::from_id_salt(egui::Id::new("browser_selector"))
                .selected_text(custom_browsers.installed_browsers.get(0).map_or("", |b| &b.name))
                .show_ui(ui, |ui| {


                    for browser in &custom_browsers.installed_browsers {
                        if ui.selectable_value(&mut self.selected_browser, Some(browser.clone()), &browser.name).clicked() {
                            logger.add_log("New Profile".to_string(), format!("Selected browser: {}", &browser.name));
                        }
                    }
                });
            ui.checkbox(&mut self.custom_profile_path_status, "");
            if self.custom_profile_path_status {
                if ui.button("Add").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.custom_profile_path = path.to_string_lossy().to_string();
                        logger.add_log("New Profile".to_string(), format!("Custom profile path selected: {}", self.custom_profile_path));
                    }
                }
            } else {
                ui.label("Default path");
            }
            ui.add(egui::TextEdit::singleline(&mut self.new_profile_name).hint_text("Enter"));
            if ui.button("CREATE").clicked() {
                if self.new_profile_name.is_empty() {
                    logger.add_log("New Profile".to_string(), "Error: Please provide a valid profile name.".to_string());
                }
                chromium_manager.add_new_profile(self.new_profile_name.clone(), "".to_string(), self.selected_browser.clone().unwrap().path);
                logger.add_log("New Profile".to_string(), format!("Profile created: {}", self.new_profile_name));
                chromium_manager.save().expect("Failed to save new profile");
            }
            ui.end_row();
        });
    }
}