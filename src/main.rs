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

mod gui {
    pub mod new_profile_section;
    pub mod single_profile_section;
    pub mod profile_list_section;
    pub mod active_profiles_section;
    pub mod proxy_manager_section;
    pub mod saved_proxies_section;
}

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
            
            gui::new_profile_section::create_new_profile_section(ui, self);

            gui::single_profile_section::single_profile_section(ui, self);

            gui::profile_list_section::profile_list_section(ui, self);

            gui::active_profiles_section::active_profiles_section(ui, self);

            gui::proxy_manager_section::proxy_manager_section(ui, self);

            gui::saved_proxies_section::saved_proxies_section(ui, self);

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
