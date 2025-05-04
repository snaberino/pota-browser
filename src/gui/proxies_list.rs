use eframe::egui;
use crate::gui::logger::LoggerPanel;
use crate::proxy::proxy_manager::{ProxyManager, ProxyConfig};

pub struct ProxiesListPanel {
    selected_proxy: ProxyConfig,
    id_to_edit: u32,
    save_changes_status: bool,
    show_username: bool,
    show_password: bool,
}

impl ProxiesListPanel {
    pub fn new() -> Self {
        Self {
            selected_proxy: ProxyConfig::default(),
            id_to_edit: 0,
            save_changes_status: false,
            show_username: false,
            show_password: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, logger: &mut LoggerPanel, proxy_manager: &mut ProxyManager) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.heading("Proxies List");

            if self.id_to_edit > 0 {
                egui::SidePanel::right("update_proxy")
                  .resizable(true)
                  .show_inside(ui, | ui| {
                    egui::Grid::new("edit").min_col_width(200.0).show(ui, | ui | {
                        ui.label("Editing:");
                        ui.label(self.selected_proxy.id.clone().to_string());
                        ui.end_row();
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.selected_proxy.name);
                        ui.end_row();
                        ui.label("Protocol");
                        ui.text_edit_singleline(&mut self.selected_proxy.protocol);
                        ui.end_row();
                        ui.label("Host");
                        ui.text_edit_singleline(&mut self.selected_proxy.host);
                        ui.end_row();
                        ui.label("Port");
                        ui.text_edit_singleline(&mut self.selected_proxy.port);
                        ui.end_row();
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.selected_proxy.username);
                        ui.end_row();
                        ui.label("Password:");
                        ui.text_edit_singleline(&mut self.selected_proxy.password);
                        ui.end_row();
                        if ui.button("Save Changes").clicked() {
                            self.save_changes_status = true;
                            // self.id_to_edit = self.selected_profile.id;
                        }
                        if ui.button("Close").clicked() {
                            self.id_to_edit = 0;
                            self.save_changes_status = false;
                        }
                    }); 
                  });
                }
    
            let mut proxy_to_remove = Vec::new();
    
            egui::Grid::new("proxies_list").show(ui, |ui| {
                ui.label("ID");
                ui.label("Name");
                ui.label("Protocol");
                ui.label("Host");
                ui.label("Port");
                ui.horizontal(|ui| {
                    ui.label("Username");
                    if ui.selectable_label(self.show_username, "👁").clicked()  {
                        self.show_username = !self.show_username;
                    };
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password");
                    if ui.selectable_label(self.show_password, "👁").clicked() {
                        self.show_password = !self.show_password;
                    };
                });
                ui.label("Remove");
                ui.label("Check");
                ui.label("Update");
                ui.end_row();
                let iter_proxies = proxy_manager.proxies.clone();
                for(index, proxy) in iter_proxies.iter().enumerate() {
                    ui.label(&proxy.id.to_string());
                    ui.label(&proxy.name);
                    ui.label(&proxy.protocol);
                    ui.label(&proxy.host);
                    ui.label(&proxy.port);
                    if self.show_username {
                        ui.label(&proxy.username);
                    } else {
                        ui.label("********");
                    }
                    if self.show_password {
                        ui.label(&proxy.password);
                    } else {
                        ui.label("********");
                    }
                    if ui.button("Remove").clicked() {
                        proxy_to_remove.push(index);
    
                    }
                    if ui.button("Check").clicked() {
                        self.selected_proxy = proxy.clone();
                        if let Err(e) = proxy_manager.start_check_proxy(self.selected_proxy.id.clone(), self.selected_proxy.clone()) {
                            logger.add_log("Proxies List".to_string(), format!("Failed to check proxy: {}. Error: {}", &self.selected_proxy.name, e));
                        } else {
                            logger.add_log("Proxies List".to_string(), format!("Checked proxy: {}", &self.selected_proxy.name));
                        }
                    }
                    if ui.button("Update").clicked() {
                        self.selected_proxy = proxy.clone();
                        self.id_to_edit = proxy.id;
                    }
                    ui.end_row();
                }
                for index in proxy_to_remove {
                    proxy_manager.remove_proxy(index);
                }
            });
        });


        if self.save_changes_status == true {
            proxy_manager.update(self.id_to_edit, self.selected_proxy.clone());
            logger.add_log("Proxies".to_string(), format!("Saved changes to proxy: {}", self.id_to_edit));
            self.save_changes_status = false;
        }
    }

}