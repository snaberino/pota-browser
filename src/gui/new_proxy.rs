use eframe::egui;
use crate::gui::logger::LoggerPanel;
use crate::proxy::proxy_manager::{ProxyManager};

pub struct NewProxyPanel {
    // Fields for the new proxy panel
    name: String,
    protocol: String,
    host: String,
    port: String,
    username: String,
    password: String,
}

impl NewProxyPanel {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            protocol: String::new(),
            host: String::new(),
            port: String::new(),
            username: String::new(),
            password: String::new(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, logger: &mut LoggerPanel, proxy_manager: &mut ProxyManager) {
        ui.heading("Add proxy");
        // Add your UI elements here
        egui::Grid::new("new_proxy").show(ui, |ui| {
            ui.label("Name");
            ui.label("Protocol");
            ui.label("Host");
            ui.label("Port");
            ui.label("Username");
            ui.label("Password");
            ui.end_row();
            ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Enter name"));
            ui.add(egui::TextEdit::singleline(&mut self.protocol).hint_text("Enter protocol"));
            ui.add(egui::TextEdit::singleline(&mut self.host).hint_text("Enter host"));
            ui.add(egui::TextEdit::singleline(&mut self.port).hint_text("Enter port"));
            ui.add(egui::TextEdit::singleline(&mut self.username).hint_text("Enter username"));
            ui.add(egui::TextEdit::singleline(&mut self.password).hint_text("Enter password"));
            if ui.button("Add Proxy").clicked() {
                proxy_manager.add_new_proxy(self.name.clone(), self.protocol.clone(), self.host.clone(), self.port.clone(), self.username.clone(), self.password.clone());
                logger.add_log("New Proxy".to_string(), format!("Added new proxy: {}. Protocol: {}. Host: {}. Port: {}. Username: {}. Password: {}", self.name, self.protocol, self.host, self.port, self.username, self.password));
            }
        });
    }
}