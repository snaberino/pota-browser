use eframe::egui;
use crate::gui::logger::LoggerPanel;
use crate::chromium::chromium_manager::{ChromiumManager, ChromiumProfile};
use crate::proxy::proxy_manager::ProxyManager;

pub struct ProfilesListPanel {
    selected_profile: ChromiumProfile,
    id_to_edit: u32,
    save_changes_status: bool,
}

impl ProfilesListPanel {
    pub fn new() -> Self {
        Self {
            selected_profile: ChromiumProfile::default(),
            id_to_edit: 0,
            save_changes_status: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, logger: &mut LoggerPanel, chromium_manager: &mut ChromiumManager, proxy_manager: &ProxyManager) {
        ui.heading("Profiles List");

        if self.id_to_edit > 0 {
            egui::SidePanel::right("update_panel")
                .resizable(true)
                // .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.heading("Update Profile");
                    egui::Grid::new("edit").min_col_width(200.0).show(ui, |ui| {
                        ui.label("Editing:");
                        ui.label(self.selected_profile.id.clone().to_string());
                        ui.end_row();
                        ui.label("Profile name:");
                        ui.text_edit_singleline(&mut self.selected_profile.name);
                        ui.end_row();
                        ui.label("Profile path:");
                        ui.text_edit_singleline(&mut self.selected_profile.profile_path);
                        ui.end_row();
                        ui.label("Browser path:");
                        ui.text_edit_singleline(&mut self.selected_profile.browser_path);
                        ui.end_row();
                        let mut debug_enabled = self.selected_profile.debugging_port != 0;
                        if ui.checkbox(&mut debug_enabled, "Enable Debugging").changed() {
                            if debug_enabled {
                                let base_port = 9222;
                                self.selected_profile.debugging_port = chromium_manager.profiles
                                    .iter()
                                    .position(|p| p.name == self.selected_profile.name)
                                    .map(|index| base_port + index as u32)
                                    .unwrap_or(base_port);
                            } else {
                                self.selected_profile.debugging_port = 0;
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label("Port:");
                            ui.add_enabled_ui(debug_enabled, |ui| {
                                ui.add(egui::DragValue::new(&mut self.selected_profile.debugging_port).speed(1));
                            });
                        });
                        ui.end_row();
                        ui.label("Headless:");
                        let mut headless_enabled = self.selected_profile.headless;
                        if ui.toggle_value(&mut headless_enabled, "headless").changed() {
                            self.selected_profile.headless = headless_enabled;
                        }
                        ui.end_row();
                        ui.label("WebRTC:");
                        ui.horizontal(|ui| {
                            if ui.selectable_label(self.selected_profile.webrtc == "default", "default").clicked() {
                                self.selected_profile.webrtc = "default".to_string();
                            }
                            if ui.selectable_label(self.selected_profile.webrtc == "fake", "fake").clicked() {
                                self.selected_profile.webrtc = "fake".to_string();
                            }
                            if ui.selectable_label(self.selected_profile.webrtc == "disabled", "disabled").clicked() {
                                self.selected_profile.webrtc = "disabled".to_string();
                            }
                        });
                        ui.end_row();
                        ui.label("Proxy:");
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("proxy")
                                .selected_text(self.selected_profile.proxy.name.clone())
                                .show_ui(ui, |ui| {
                                    for proxy in proxy_manager.proxies.iter() {
                                        ui.selectable_value(&mut self.selected_profile.proxy, proxy.clone(), &proxy.name);
                                    }
                                });
                            if ui.button("Set Proxy").clicked() {
                                chromium_manager.set_proxy(self.id_to_edit, self.selected_profile.proxy.clone()).expect("error setting proxy");
                            }
                        });
                    });

                    if ui.button("Save Changes").clicked() {
                        self.save_changes_status = true;
                        // self.id_to_edit = self.selected_profile.id;
                    }

                    if ui.button("Close").clicked() {
                        self.id_to_edit = 0;
                        self.save_changes_status = false;
                    }
                });
        }

        let mut profile_to_remove = Vec::new();
        // let mut profile_to_remove = Vec::u;

        egui::Grid::new("profiles_list_grid").show(ui, |ui| {
            ui.label("ID");
            ui.label("Profile Name");
            ui.label("Start");
            ui.label("Stop");
            ui.label("Remove");
            ui.label("Settings");
            ui.end_row();

            let iter_profiles = chromium_manager.profiles.clone();
            for (index, profile) in iter_profiles.iter().enumerate() {
                ui.label((1+index).to_string());
                ui.label(&profile.name);
                if ui.button("Start").clicked() {
                    if let Err(e) = chromium_manager.start(profile.clone()) {
                        logger.add_log("Profiles List".to_string(), format!("Failed to start profile: {}. Error: {}", &profile.name, e));
                    } else {
                        logger.add_log("Profiles List".to_string(), format!("Started profile: {}", &profile.name));
                    }
                }
                if ui.button("Stop").clicked() {
                    if let Err(e) = chromium_manager.stop(profile.name.as_str()) {
                        logger.add_log("Profiles List".to_string(), format!("Failed to stop profile: {}. Error: {}", &profile.name, e));
                    } else {
                        logger.add_log("Profiles List".to_string(), format!("Stopped profile: {}", &profile.name));
                    }
                }
                if ui.button("Remove").clicked() {
                    logger.add_log("Profiles List".to_string(), format!("Removed profile: {}", &profile.name));
                    profile_to_remove.push(index);
                    // profile_to_remove.push(&profile.id);

                    // TODO: Delete the profile directory
                }
                if ui.button("Update").clicked() {
                    self.selected_profile = profile.clone();
                    self.id_to_edit = profile.id;
                }
                ui.end_row();
            }
        });

        for index in profile_to_remove {
            chromium_manager.remove(index);
            chromium_manager.save().unwrap();
        }

        if self.save_changes_status == true {
            chromium_manager.update( self.id_to_edit, self.selected_profile.clone()).unwrap();
            logger.add_log("Profiles List".to_string(), format!("Updated profile: {}", self.id_to_edit));
        }
    }
}
