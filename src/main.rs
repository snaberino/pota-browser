pub mod chrome;
pub mod proxy_manager;
pub mod websocket;

use proxy_manager::ProxiesConfig;
use proxy_manager::ProxyConfig;

use chrome::ChromeProfile;
use chrome::ChromeProfiles;

use eframe::egui;

use tokio::runtime::Builder;

fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.viewport.resizable = Some(true);
    eframe::run_native(
        "pota browser",
        options,
        Box::new(|_cc| Ok(Box::new(ProfileManager::default()))),
    )
}

struct ProfileManager {
    profiles: ChromeProfiles,
    open_profiles: ChromeProfiles,
    selected_profile: ChromeProfile,
    new_profile_name: String, // Per creare nuovi profili
    proxy_configs: ProxiesConfig,
    proxy : ProxyConfig,
    selected_proxy: ProxyConfig,
    log_message : String, // Per rederizzare i messaggi di errore o successo, sulla GUI. Adotto un solo elemento della gui per stampare tutti i messaggi per l'utente.
}

impl Default for ProfileManager {
    fn default() -> Self {
        // Loading existing profiles
        let profiles: ChromeProfiles = chrome::load_profile_configs();
        let selected_profile = profiles.get(0).cloned().unwrap_or_else(|| {
            ChromeProfile {
                name: "Default".to_string(),
                path: chrome::get_profile_dir("Default"),
                debugging_port: 0,
                headless: false,
                proxy: ProxyConfig {
                    proxy_type: "socks5".to_string(),
                    proxy_name: String::new(),
                    proxy_host: String::new(),
                    proxy_port: String::new(),
                    proxy_username: String::new(),
                    proxy_password: String::new(),
                },
            }
        });

        // Loading existing proxy configs	
        let proxy_configs: ProxiesConfig = proxy_manager::load_proxy_configs();
        let selected_proxy = proxy_configs.get(0).cloned().unwrap_or_else(|| {
            ProxyConfig {
                proxy_name: "Default".to_string(),
                proxy_type: "socks5".to_string(),
                proxy_host: "host".to_string(),
                proxy_port: "port".to_string(),
                proxy_username: "username".to_string(),
                proxy_password: "password".to_string(),
            }
        });

        Self {
            profiles,
            open_profiles: ChromeProfiles::new(),
            selected_profile,
            new_profile_name: String::new(),
            proxy_configs,
            proxy: ProxyConfig {
                proxy_type: "socks5".to_string(),
                proxy_name: String::new(),
                proxy_host: String::new(),
                proxy_port: String::new(),
                proxy_username: String::new(),
                proxy_password: String::new(),
            },
            selected_proxy,
            log_message: String::new(),
        }
    }
}

impl eframe::App for ProfileManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut proxy_configs = proxy_manager::load_proxy_configs();

        egui::CentralPanel::default().show(ctx, |ui| {
            //
            // New profile section
            //
            ui.label("Create a new profile:");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_profile_name)); 

                if ui.button("CREATE").clicked() {
                    if !self.new_profile_name.trim().is_empty() {
                        let new_profile = ChromeProfile {
                            name: self.new_profile_name.clone(),
                            path: chrome::get_profile_dir(&self.new_profile_name.clone()),
                            debugging_port: 0,
                            headless: true,
                            proxy: ProxyConfig {
                                proxy_type: "socks5".to_string(),
                                proxy_name: String::new(),
                                proxy_host: String::new(),
                                proxy_port: String::new(),
                                proxy_username: String::new(),
                                proxy_password: String::new(),
                            },
                        };
                        self.profiles.push(new_profile.clone());
                        match chrome::create_new_profile(new_profile.clone()) {
                            Ok(_) => self.log_message = format!("Profile {} created successfully.", new_profile.name),
                            Err(e) => self.log_message = format!("Errore: {}", e),
                        }
                        chrome::save_profile_configs(&self.profiles);
                        self.new_profile_name.clear();
                        self.log_message = format!("Profile {} created successfully.", new_profile.name);
                    } else {
                        self.log_message = format!("Error: provide a valid name.");
                    }
                }
            });
            ui.separator();

            //
            // Single profile section selection and edit actions
            //
            ui.horizontal(|ui| {
                ui.label("PROFILE SELECTOR");
                egui::ComboBox::from_id_salt(egui::Id::new("profile_selector"))
                .selected_text(&self.selected_profile.name)
                .show_ui(ui, |ui| {
                    for profile in &self.profiles {
                        ui.selectable_value(
                            &mut self.selected_profile,
                            profile.clone(),
                            &profile.name,
                        );
                    }
                });
                if self.profiles.is_empty() {
                    ui.add_enabled(false, egui::Button::new("OPEN"));
                    self.log_message = "No profile found.".to_string();
                } else {
                    if ui.button("OPEN").clicked() {
                        match chrome::open_chrome(self.selected_profile.clone()) {
                            Ok(_) => {
                                self.log_message = format!("Profile {} opened successfully.", self.selected_profile.name);
                                self.open_profiles.push(self.selected_profile.clone());
                            }
                            Err(e) => self.log_message = format!("Error: {}", e),
                        };
                    }
                    // Debugging port checkbox, allow user to enable or disable debugging port for the selected profile
                    let mut debug_enabled = self.selected_profile.debugging_port != 0;
                    if ui.checkbox(&mut debug_enabled, "Debug Mode").changed() {
                        self.selected_profile.debugging_port = if debug_enabled { 9222 } else { 0 };
                        // Update the profile in the profiles list
                        if let Some(profile) = self.profiles.iter_mut().find(|p| p.name == self.selected_profile.name) {
                            profile.debugging_port = self.selected_profile.debugging_port;
                        }
                        chrome::save_profile_configs(&self.profiles);
                        self.log_message = format!("Debugging port for profile {} set to {}.", self.selected_profile.name, self.selected_profile.debugging_port);
                    }
                    // Checkbox per headless mode, allow user to enable or disable headless mode for the selected profile
                    let mut headless_enabled = self.selected_profile.headless;
                    if ui.checkbox(&mut headless_enabled, "Headless Mode").changed() {
                        self.selected_profile.headless = if headless_enabled { true } else { false };
                        if let Some(profile) = self.profiles.iter_mut().find(|p| p.name == self.selected_profile.name) {
                            profile.headless = self.selected_profile.headless;
                        }
                        chrome::save_profile_configs(&self.profiles);
                        self.log_message = format!("Headless mode for profile {} set to {}.", self.selected_profile.name, self.selected_profile.headless);
                    }
                }
            }); //  horizontal

            ui.horizontal(|ui| {
                ui.label("Proxy:");
                egui::ComboBox::from_id_salt(egui::Id::new("proxy_selector"))
                .selected_text(&self.selected_proxy.proxy_name)
                .show_ui(ui, |ui| {
                    for proxy in &mut proxy_configs {
                        ui.selectable_value(&mut self.selected_proxy, proxy.clone(), &proxy.proxy_name);
                    }
                });
                if ui.button("Set Proxy").clicked() {
                    self.selected_profile.proxy = self.selected_proxy.clone();
                    println!("Proxy selected: {:?}", self.selected_proxy);
                    println!("Profile selected: {:?}", self.selected_profile);
                }
            }); // horizontal

            ui.separator();

            //
            // Section for list of profiles
            //
            ui.label("PROFILE LIST");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for profile in &mut self.profiles {
                    ui.horizontal(|ui| {
                        ui.label(&profile.name);
                        if ui.button("START").clicked() {
                            profile.headless = false;
                            match chrome::open_chrome(profile.clone()) {
                                Ok(_) => {
                                    self.log_message = format!("Profile {} opened successfully.", profile.name);
                                    self.open_profiles.push(profile.clone());
                                }
                                Err(e) => self.log_message = format!("Error: {}", e),
                            };
                            // self.open_profiles.push(profile.clone());
                            // self.log_message = format!("Profile {} opened successfully.", profile.name);
                        }
                        if ui.button("BOT").clicked() {
                            let ws_url = websocket::get_ws_url(&profile.clone());
                            println!("WebSocket URL: {:?}", ws_url);
                        }
                        
                    }); // horizontal
                }
            });
            ui.separator();

            //
            // Section for active profiles
            //
            ui.label("Active Profile:");
            for profile in self.open_profiles.clone() {
                ui.horizontal(|ui| {
                    ui.label(&profile.name);
                    if ui.button("CLOSE").clicked() {
                        match chrome::close_chrome(&profile.name) {
                            Ok(_) => self.log_message = format!("Profile {} closed successfully.", profile.name),
                            Err(e) => self.log_message = format!("Error: {}", e),
                        }
                        // chrome::close_chrome(&profile.name);
                        self.open_profiles.retain(|p| p.name != profile.name); // Rimuovi il profilo dalla lista
                    }
                });
            }
            ui.separator();

            //
            // Section for proxy manager
            // New proxy
            ui.label("Proxy Manager");
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("")
                .selected_text(&self.proxy.proxy_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.proxy.proxy_type, "socks5".to_string(), "socks5");
                    ui.selectable_value(&mut self.proxy.proxy_type, "http".to_string(), "http");
                    ui.selectable_value(&mut self.proxy.proxy_type, "https".to_string(), "https");
                });
            });
            ui.horizontal(|ui| {
                ui.label("name:");
                ui.text_edit_singleline(&mut self.proxy.proxy_name);
            });
            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.text_edit_singleline(&mut self.proxy.proxy_host);
            });
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(&mut self.proxy.proxy_port);
            });
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut self.proxy.proxy_username);
            });
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.text_edit_singleline(&mut self.proxy.proxy_password);
            });

            if ui.button("Test Proxy").clicked() {
                let proxy_url;
                if self.proxy.proxy_username.is_empty(){
                    proxy_url = format!(
                        "{}://{}:{}",
                        self.proxy.proxy_type,
                        self.proxy.proxy_host,
                        self.proxy.proxy_port
                    );
                } else {
                    proxy_url = format!(
                        "{}://{}:{}@{}:{}",
                        self.proxy.proxy_type,
                        self.proxy.proxy_username,
                        self.proxy.proxy_password,
                        self.proxy.proxy_host,
                        self.proxy.proxy_port
                    );
                }
                println!("Proxy URL: {}", proxy_url); //debugging

                let rt = Builder::new_current_thread().enable_all().build().unwrap();
                rt.block_on(async {
                    match proxy_manager::test_proxy(&proxy_url).await {
                        Ok((ip, country)) => {
                            self.log_message = format!("IP: {}, Country: {}", ip, country);
                        }
                        Err(e) => {
                            self.log_message = format!("Error: {}", e);
                        }
                    }
                });
            }

            if ui.button("SAVE PROXY").clicked() {
                self.proxy_configs.push(
                    ProxyConfig {
                        proxy_type: self.proxy.proxy_type.clone(),
                        proxy_name: self.proxy.proxy_name.clone(),
                        proxy_host: self.proxy.proxy_host.clone(),
                        proxy_port: self.proxy.proxy_port.clone(),
                        proxy_username: self.proxy.proxy_username.clone(),
                        proxy_password: self.proxy.proxy_password.clone(),
                    }
                );
                proxy_manager::save_proxy_configs(&self.proxy_configs);
                self.log_message = format!("Proxy {} added successfully.", self.proxy.proxy_name);
            }

            // if ui.button("Salva proxy-config").clicked() {
            //     proxy_manager::save_proxy_configs(&self.proxy_configs);
            //     self.log_message = format!("Configurazione proxy salvata con successo.");
            // }

            ui.separator();
            
            //
            // Section for list of saved proxies configs
            //
            ui.horizontal(|ui|{
                ui.vertical(|ui|{
                    ui.label("name");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.proxy_name));
                    }
                });
                ui.vertical(|ui|{
                    ui.label("type");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.proxy_type));
                    }
                });
                ui.vertical(|ui|{
                    ui.label("host");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.proxy_host));
                    }
                });
                ui.vertical(|ui|{
                    ui.label("port");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.proxy_port));
                    }
                });
                ui.vertical(|ui| {
                    ui.label("username");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.proxy_username));
                    }
                });
                ui.vertical(|ui| {
                    ui.label("password");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.proxy_password));
                    }
                });
                ui.vertical(|ui| {
                    ui.label("Test");
                    for proxy in &mut proxy_configs {
                        if ui.button("Test Proxy").clicked() {
                            let proxy_url = format!(
                                "{}://{}:{}@{}:{}",
                                proxy.proxy_type,
                                proxy.proxy_username,
                                proxy.proxy_password,
                                proxy.proxy_host,
                                proxy.proxy_port
                            );
                            println!("Proxy URL: {}", proxy_url);
            
                            let rt = Builder::new_current_thread().enable_all().build().unwrap();
                            rt.block_on(async {
                                match proxy_manager::test_proxy(&proxy_url).await {
                                    Ok((ip, country)) => {
                                        self.log_message = format!("IP: {}, Country: {}", ip, country);
                                    }
                                    Err(e) => {
                                        self.log_message = format!("Errore: {}", e);
                                    }
                                }
                            });
                        }
                    }
                });
            });
            ui.separator();

            // Renderizzo i messaggi di log
            if !self.log_message.is_empty() {
                if self.log_message.starts_with("Error") {
                    ui.colored_label(egui::Color32::RED, &self.log_message);
                } else {
                    ui.colored_label(egui::Color32::GREEN, &self.log_message);
                }
            }
        });    
    }
}
