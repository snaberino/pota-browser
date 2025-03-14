use crate::ProfileManager;
use eframe::egui;

pub fn proxy_manager_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    ui.label("Proxy Manager");
    ui.horizontal(|ui| {
        egui::ComboBox::from_label("")
            .selected_text(&manager.proxy.proxy_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut manager.proxy.proxy_type, "socks5".to_string(), "socks5");
                ui.selectable_value(&mut manager.proxy.proxy_type, "http".to_string(), "http");
                ui.selectable_value(&mut manager.proxy.proxy_type, "https".to_string(), "https");
            });
    });
    ui.horizontal(|ui| {
        ui.label("name:");
        ui.text_edit_singleline(&mut manager.proxy.proxy_name);
    });
    ui.horizontal(|ui| {
        ui.label("Host:");
        ui.text_edit_singleline(&mut manager.proxy.proxy_host);
    });
    ui.horizontal(|ui| {
        ui.label("Port:");
        ui.text_edit_singleline(&mut manager.proxy.proxy_port);
    });
    ui.horizontal(|ui| {
        ui.label("Username:");
        ui.text_edit_singleline(&mut manager.proxy.proxy_username);
    });
    ui.horizontal(|ui| {
        ui.label("Password:");
        ui.text_edit_singleline(&mut manager.proxy.proxy_password);
    });

    if ui.button("Test Proxy").clicked() {
        let proxy_url;
        if manager.proxy.proxy_username.is_empty() {
            proxy_url = format!(
                "{}://{}:{}",
                manager.proxy.proxy_type,
                manager.proxy.proxy_host,
                manager.proxy.proxy_port
            );
        } else {
            proxy_url = format!(
                "{}://{}:{}@{}:{}",
                manager.proxy.proxy_type,
                manager.proxy.proxy_username,
                manager.proxy.proxy_password,
                manager.proxy.proxy_host,
                manager.proxy.proxy_port
            );
        }
        println!("Proxy URL: {}", proxy_url); //debugging

        crate::proxy_manager::start_check_proxy(manager.proxy.clone(), manager.proxy_configs.clone());
    }

    if ui.button("SAVE PROXY").clicked() {
        manager.proxy_configs.push(
            crate::ProxyConfig {
                proxy_type: manager.proxy.proxy_type.clone(),
                proxy_name: manager.proxy.proxy_name.clone(),
                proxy_host: manager.proxy.proxy_host.clone(),
                proxy_port: manager.proxy.proxy_port.clone(),
                proxy_username: manager.proxy.proxy_username.clone(),
                proxy_password: manager.proxy.proxy_password.clone(),

                country: String::new(),
                lang_arg: String::new(),
                accept_language_arg: String::new(),
                last_ip: String::new(),
                used_ips: vec![],
            }
        );
        crate::proxy_manager::save_proxy_configs(&manager.proxy_configs);
        manager.log_message = format!("Proxy {} added successfully.", manager.proxy.proxy_name);
    }

    ui.separator();
}