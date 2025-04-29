use eframe::egui;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;

use crate::gui::logger::LoggerPanel;
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CustomBrowser {
    pub name: String,
    pub path: String,
}
#[derive(Serialize, Deserialize)]
pub struct CustomBrowserPanel {
    pub installed_browsers: Vec<CustomBrowser>,
    pub new_browser_name: String,
}

impl CustomBrowserPanel {
    pub fn new() -> Self {
        if let Ok(file) = File::open("custom_browsers.json") {
            if let Ok(mut browser) = serde_json::from_reader::<_, CustomBrowserPanel>(&file) {
                browser.new_browser_name = "".to_string();
                return browser;
            }
            else {
                // File doesn't exist, create it
                let _ = File::create("custom_browsers.json").expect("Failed to create custom_browsers.json");
            }
        } else {
            println!("No custom browsers found");
        }
        Self {
            installed_browsers: Vec::new(),
            new_browser_name: String::new(),

            // logger: LoggerPanel::new(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, logger: &mut LoggerPanel) {
        ui.heading("Custom Browsers");
        egui::Grid::new("custom_browsers_grid").show(ui, |ui| {
            ui.label("Name");
            ui.label("Actions");
            ui.end_row();
            ui.add(egui::TextEdit::singleline(&mut self.new_browser_name).hint_text("Enter browser name"));
            if ui.button("Add executable").clicked() {
                if let Some(path) = rfd::FileDialog::new().set_title("Select Browser Executable").pick_file() {
                    let new_browser = CustomBrowser {
                        name: self.new_browser_name.clone(),
                        path: path.display().to_string(),
                    };
                    self.installed_browsers.push(new_browser.clone());
                    self.save().unwrap();
                    logger.add_log("Custom Browsers".to_string(), format!("Added {} at path: {}", new_browser.name, &new_browser.path));
                }
            }
            ui.end_row();
        });
        ui.separator();
        ui.heading("Installed Browsers");
        let mut browser_to_remove = Vec::new();
        egui::Grid::new("installed_browsers_grid").show(ui, |ui| {
            ui.label("Name");
            ui.label("Path");
            ui.label("Actions");
            ui.end_row();

            for (index, browser) in self.installed_browsers.iter().enumerate() {
                ui.label(&browser.name);
                ui.label(&browser.path);
                if ui.button("Delete").clicked() {
                    browser_to_remove.push(index);
                }
                ui.end_row();
            }
            });
        for index in browser_to_remove {
            self.installed_browsers.remove(index);
            self.save().unwrap();
            logger.add_log("Custom Browsers".to_string(), format!("Removed browser at index: {}", index));
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create("custom_browsers.json")?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }
}