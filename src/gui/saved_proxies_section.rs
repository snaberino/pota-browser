use crate::ProfileManager;
use eframe::egui;

pub fn saved_proxies_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    let proxy_configs_clone = manager.proxy_configs.clone();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("name");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.proxy_name));
            }
        });
        ui.vertical(|ui| {
            ui.label("type");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.proxy_type));
            }
        });
        ui.vertical(|ui| {
            ui.label("host");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.proxy_host));
            }
        });
        ui.vertical(|ui| {
            ui.label("port");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.proxy_port));
            }
        });
        ui.vertical(|ui| {
            ui.label("username");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.proxy_username));
            }
        });
        ui.vertical(|ui| {
            ui.label("password");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.proxy_password));
            }
        });
        ui.vertical(|ui| {
            ui.label("country");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.country));
            }
        });
        ui.vertical(|ui| {
            ui.label("last_ip");
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", &mut proxy.last_ip));
            }
        });
        ui.vertical(|ui| {
            ui.label("Test");
            for proxy in &mut manager.proxy_configs {
                if ui.button("CHECK").clicked() {
                    let proxy_url = format!(
                        "{}://{}:{}@{}:{}",
                        proxy.proxy_type,
                        proxy.proxy_username,
                        proxy.proxy_password,
                        proxy.proxy_host,
                        proxy.proxy_port
                    );
                    println!("Proxy URL: {}", proxy_url);
                    let new_handle = crate::proxy_manager::start_check_proxy(proxy.clone(), proxy_configs_clone.clone());
                    // let new_handle = crate::proxy_manager::start_check_proxy(proxy.clone(), manager.proxy_configs.clone());
                    manager.check_handles.push(new_handle);
                    manager.log_message = format!("Checking proxy {} in background...", proxy.proxy_name);
                }
            }
        });
    });
    ui.separator();
}