use crate::ProfileManager;
use eframe::egui;

pub fn saved_proxies_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    let proxy_configs_clone = manager.proxy_configs.clone();
    ui.label("Saved Proxies:");
    ui.separator();

    egui::Grid::new("proxy_table")
        .striped(true)
        .show(ui, |ui| {
            // Table headers
            ui.label("Name");
            ui.label("Type");
            ui.label("Host");
            ui.label("Port");
            ui.label("Username");
            ui.label("Password");
            ui.label("Country");
            ui.label("Last IP");
            ui.label("Test");
            ui.end_row();

            // Table rows
            for proxy in &mut manager.proxy_configs {
                ui.label(format!("{}", proxy.proxy_name));
                ui.label(format!("{}", proxy.proxy_type));
                ui.label(format!("{}", proxy.proxy_host));
                ui.label(format!("{}", proxy.proxy_port));
                ui.label(format!("{}", proxy.proxy_username));
                ui.label(format!("{}", proxy.proxy_password));
                ui.label(format!("{}", proxy.country));
                ui.label(format!("{}", proxy.last_ip));

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
                    manager.check_handles.push(new_handle);
                    manager.log_message = format!("Checking proxy {} in background...", proxy.proxy_name);
                }
                ui.end_row();
            }
        });

    ui.separator();
}