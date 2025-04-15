use eframe::egui;
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct LoggerPanel {
    logger_status: bool,
    logger_message: HashMap<String, Vec<String>>
}
impl LoggerPanel {
    pub fn new() -> Self {
        Self {
            logger_status: false,
            logger_message: HashMap::new(),
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        for (log_id, logs) in &self.logger_message {
            ui.collapsing(format!("{}", log_id), |ui| {
               for log in logs {
                   ui.label(log);
               }
            });
        }
    }

    pub fn add_log(&mut self, log_id: String, log_message: String) {
        self.logger_message.entry(log_id).or_insert(Vec::new()).push(log_message);
    }
}