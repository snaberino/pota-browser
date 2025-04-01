use crate::ProfileManager;
use crate::proxy_manager::ProxyConfig;
use eframe::egui;

pub fn saved_proxies_section(ui: &mut egui::Ui, manager: &mut ProfileManager) {
    let proxy_configs_clone = manager.proxy_configs.clone();
    ui.heading("Saved Proxies:");

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
            ui.label("Delete");
            ui.end_row();

            // Table rows
            let mut indices_to_remove = Vec::new();
            for (index, proxy) in manager.proxy_configs.iter_mut().enumerate() {
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
                    let new_handle = ProxyConfig::start_check_proxy(proxy.clone(), proxy_configs_clone.clone());
                    manager.check_handles.push(new_handle);
                    manager.log_message = format!("Checking proxy {} in background...", proxy.proxy_name);
                }

                if ui.button("DELETE").clicked() {
                    indices_to_remove.push(index);
                    manager.log_message = format!("Deleted proxy {}", proxy.proxy_name);
                    // break; // Break to avoid iterator invalidation
                }
                ui.end_row();
            }

            
            
        // Remove proxies after the loop to avoid mutable borrow conflicts
        for index in indices_to_remove.into_iter().rev() {
            manager.proxy_configs.remove(index);
        }

        });

    ui.separator();
}