pub mod chrome;
pub mod proxy_manager;
pub mod websocket;
pub mod fingerprint_manager;

use tokio::task::JoinHandle;
use futures::future::FutureExt;

use proxy_manager::ProxiesConfig;
use proxy_manager::ProxyConfig;

use chrome::ChromeProfile;
use chrome::ChromeProfiles;

use fingerprint_manager::FingerprintManager;
use fingerprint_manager::SingleFingerprint;

use eframe::egui;

use tokio::runtime::Builder;

fn main() -> Result<(), eframe::Error> {
    // Initialize the Tokio runtime
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let mut options = eframe::NativeOptions::default();
        options.viewport.resizable = Some(true);
        eframe::run_native(
            "pota browser",
            options,
            Box::new(|_cc| Ok(Box::new(ProfileManager::default()))),
        )
    })
}

struct ProfileManager {
    profiles: ChromeProfiles, // All the profiles
    open_profiles: ChromeProfiles, // Opened profiles in current runtime
    selected_profile: ChromeProfile, // Selected profile for dropdown menu
    new_profile_name: String, // Need it in order to create a new profile
    proxy_configs: ProxiesConfig, // All the proxy configs
    selected_proxy: ProxyConfig, // Selected proxy for dropdown menu
    proxy : ProxyConfig, // Proxy struct in order to create a new proxy
    log_message : String, // Render log messages

    check_handles: Vec<JoinHandle<Result<ProxyConfig, String>>>, // Background handle for checking proxies

    // Fingerprints zone
    fingerprint_manager: FingerprintManager, // Variable for storing all the information in order to generate a fingerprint
    single_fingerprint: SingleFingerprint, // Variable for storing a single fingerprint to inject in profiles
    selected_os_list: Vec<String>, // To handle dropdown menu for OS selection
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
                proxy: ProxyConfig::new(),
                webrtc: String::new(),
                fingerprint: SingleFingerprint {
                    os_type: String::new(),
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

                country: String::new(),
                lang_arg: String::new(),
                accept_language_arg: String::new(),
                last_ip: String::new(),
                used_ips: vec![],
            }
        });

        // Loading fingerprint manager
        let fingerprint_manager: FingerprintManager = fingerprint_manager::load_fingerprint_manger();
        let selected_os_list: Vec<String> = fingerprint_manager.os_type[0].clone();

        Self {
            profiles,
            open_profiles: ChromeProfiles::new(),
            selected_profile,
            new_profile_name: String::new(),
            proxy_configs,
            proxy: ProxyConfig::new(),
            selected_proxy,
            log_message: String::new(),

            check_handles: Vec::new(),

            fingerprint_manager,
            single_fingerprint: SingleFingerprint {
                os_type: String::new(),
            },
            selected_os_list,
        }
    }
}

impl eframe::App for ProfileManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Polling the check_handles to see if any of them are done
        let mut new_handles = Vec::new();
        for mut handle in self.check_handles.drain(..) {
            if let Some(done) = std::pin::Pin::new(&mut handle).now_or_never() {
                match done {
                    Ok(Ok(proxy_config)) => {
                        self.log_message = format!(
                            "Proxy {} checked successfully. Last IP: {}",
                            proxy_config.proxy_name, proxy_config.last_ip
                        );
                    }
                    Ok(Err(e)) => {
                        self.log_message = format!("Error while checking proxy: {}", e);
                    }
                    Err(join_err) => {
                        self.log_message = format!("Tokio Join Error: {}", join_err);
                    }
                }
            } else {
                new_handles.push(handle);
            }
        }
        self.check_handles = new_handles;

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
                            proxy: ProxyConfig::new(),
                            webrtc: String::new(),
                            fingerprint: SingleFingerprint {
                                os_type: String::new(),
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
            ui.label("PROFILE CONFIGURATION");
            ui.horizontal(|ui| {
                ui.label("Select profile");
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
                ui.label("Proxy");
                egui::ComboBox::from_id_salt(egui::Id::new("proxy_selector"))
                .selected_text(&self.selected_proxy.proxy_name)
                .show_ui(ui, |ui| {
                    for proxy in &mut proxy_configs {
                        ui.selectable_value(&mut self.selected_proxy, proxy.clone(), &proxy.proxy_name);
                    }
                });
                if ui.button("Set Proxy").clicked() {
                    self.selected_profile.proxy = self.selected_proxy.clone();
                    self.log_message = format!("Proxy {} set for profile {}.", self.selected_proxy.proxy_name, self.selected_profile.name);
                    println!("Proxy selected: {:?}", self.selected_proxy);
                    println!("Profile selected: {:?}", self.selected_profile);
                }
            }); // horizontal

            ui.label("FINGERPRINT CONFIGURATION");
            ui.horizontal(|ui|{
                ui.label("OS");
                egui::ComboBox::from_id_salt(egui::Id::new("os_selector"))
                    .selected_text(&self.single_fingerprint.os_type)
                        .show_ui(ui, |ui| {
                            ui.horizontal(|ui| { 
                                if ui.button("Windows").clicked() {
                                    self.selected_os_list = self.fingerprint_manager.os_type[0].clone();
                                    println!("List A selected"); 
                                }
                                if ui.button("Mac").clicked() {
                                    self.selected_os_list = self.fingerprint_manager.os_type[1].clone();
                                    println!("List B selected");
                                }
                            });

                            for os_type in &self.selected_os_list {
                                ui.selectable_value(&mut self.single_fingerprint.os_type, os_type.clone(), os_type);
                            }

                        });
            }); // horizontal

            ui.horizontal(|ui| {
                // Dropdown menu for WebRTC IP handling
                // Store the old value
                let old_webrtc = self.selected_profile.webrtc.clone();
                ui.label("WebRTC");
                egui::ComboBox::from_id_salt(egui::Id::new("webrtc_spoofing_selector"))
                    .selected_text(&self.selected_profile.webrtc)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_profile.webrtc, "default".to_string(), "default");
                        ui.selectable_value(&mut self.selected_profile.webrtc, "fake".to_string(), "fake");
                        ui.selectable_value(&mut self.selected_profile.webrtc, "block".to_string(), "block");
                    });
                // Update only if changed
                if old_webrtc != self.selected_profile.webrtc {
                    if let Some(profile) = self.profiles.iter_mut().find(|p| p.name == self.selected_profile.name) {
                        profile.webrtc = self.selected_profile.webrtc.clone();
                        chrome::save_profile_configs(&self.profiles);
                        self.log_message = format!(
                            "WebRTC spoofing for profile {} set to {}.",
                            self.selected_profile.name,
                            self.selected_profile.webrtc
                        );
                    }
                }
            });

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

                proxy_manager::start_check_proxy(self.proxy.clone(), self.proxy_configs.clone());
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

                        country: String::new(),
                        lang_arg: String::new(),
                        accept_language_arg: String::new(),
                        last_ip: String::new(),
                        used_ips: vec![],
                    }
                );
                proxy_manager::save_proxy_configs(&self.proxy_configs);
                self.log_message = format!("Proxy {} added successfully.", self.proxy.proxy_name);
            }

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
                    ui.label("country");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.country));
                    }
                });
                ui.vertical(|ui| {
                    ui.label("last_ip");
                    for proxy in &mut proxy_configs {
                        ui.label(format!("{}", &mut proxy.last_ip));
                    }
                });
                ui.vertical(|ui| {
                    ui.label("Test");
                    for proxy in &mut proxy_configs {
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

                            let new_handle = proxy_manager::start_check_proxy(proxy.clone(), self.proxy_configs.clone());
                            self.check_handles.push(new_handle);
                            self.log_message = format!("Checking proxy {} in background...", proxy.proxy_name);
                            
                        }
                    }
                });
            });
            ui.separator();

            // Rendering log messages
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
