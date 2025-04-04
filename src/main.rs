mod gui;
mod chromium;

pub mod proxy_manager;
pub mod fingerprint_manager;

use proxy_manager::{ProxiesConfig, ProxyConfig, load_proxy_configs };
use chromium::chromium::{ ChromiumProfile, ChromiumProfiles };
use fingerprint_manager::{ FingerprintManager, SingleFingerprint };
use gui::new_profile_section::Browsers;

use futures::future::FutureExt;

use std::collections::HashMap;

use tokio::task::JoinHandle;
use tokio::runtime::Builder;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Initialize the Tokio runtime
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        // let mut new_options= eframe::WindowAttributes::default();
        let mut options = eframe::NativeOptions::default();
        options.viewport.resizable = Some(true);
        options.centered = true;
        // options.decorated = false;
        eframe::run_native(
            "pota browser",
            options,
            Box::new(|_cc| Ok(Box::new(ProfileManager::default()))),
        )
    })
}

pub struct ProfileManager {

    // CORE THINGS

    installed_browsers: Vec<Browsers>, // Variable for storing all the installed browsers
    profiles: ChromiumProfiles, // Variable for storing all the profiles
    open_profiles: ChromiumProfiles, // Opened profiles in current runtime
    proxy_configs: ProxiesConfig, // Variable for storing all the proxy configs
    proxy : ProxyConfig, // Proxy struct in order to create a new proxy
    check_handles: Vec<JoinHandle<Result<ProxyConfig, String>>>, // Background handle for checking proxies

    // GUI THINGS

    // Checkboxes
    use_custom_path: bool,
    custom_profile_path: String,

    // Edit text fields
    custom_browser_name: String, // Custom browser name
    new_profile_name: String, // Need it in order to create a new profile

    // Dropdown menu
    selected_browser_path: String, // Selected browser path for dropdown menu
    selected_profile: ChromiumProfile, // Selected profile for dropdown menu
    selected_proxy: ProxyConfig, // Selected proxy for dropdown menu
    selected_section: String, // Selected section in the GUI

    // Log messages
    cdp_log_messages: HashMap<String, Vec<String>>,
    log_message: String, // Render log messages

    // FINGERPRINT THINGS

    fingerprint_manager: FingerprintManager, // Variable for storing all the information in order to generate a fingerprint
    single_fingerprint: SingleFingerprint, // Variable for storing a single fingerprint to inject in profiles
    selected_os_list: Vec<String>, // To handle dropdown menu for OS selection
}

impl Default for ProfileManager {
    fn default() -> Self {
        // Loading existing installed browsers
        let mut installed_browsers = gui::new_profile_section::discover_installed_browsers();

        // Loading custom browsers
        let custom_browsers: Vec<Browsers> = gui::new_profile_section::load_custom_browsers();
        // Adding custom browsers avoiding duplicates with installed ones
        for custom_browser in custom_browsers {
            if !installed_browsers.iter().any(|browser| browser.path == custom_browser.path) {
                installed_browsers.push(custom_browser);
            }
        }

        // Loading existing profiles
        let profiles = ChromiumProfile::load_profile_configs();
        let selected_profile = profiles.get(0).cloned().unwrap_or_else(|| { ChromiumProfile::new("-") });

        // Loading existing proxy configs	
        let proxy_configs: ProxiesConfig = load_proxy_configs();
        let selected_proxy = proxy_configs.get(0).cloned().unwrap_or_else(|| { ProxyConfig::new() });

        // Loading fingerprint manager just for reference, at the moment nothing done
        let fingerprint_manager: FingerprintManager = fingerprint_manager::load_fingerprint_manger();
        let selected_os_list: Vec<String> = fingerprint_manager.os_type[0].clone();

        Self {
            installed_browsers,
            custom_browser_name: String::new(),

            use_custom_path: false,
            custom_profile_path: String::new(),

            new_profile_name: String::new(),
            selected_browser_path: String::new(),

            profiles,
            open_profiles: ChromiumProfiles::new(),
            selected_profile,
            
            proxy_configs,
            proxy: ProxyConfig::new(),
            selected_proxy,
            check_handles: Vec::new(),

            fingerprint_manager,
            single_fingerprint: SingleFingerprint {
                os_type: String::new(),
            },
            selected_os_list,
            selected_section: "profiles_manager".to_string(), // Default selected section

            cdp_log_messages: HashMap::new(),
            log_message: String::new(),
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
                        self.proxy_configs = load_proxy_configs();
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

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Pota Browser");

            ui.separator();

            // Side panel buttons for navigation
            if ui.button("NEW PROFILE").clicked() {
                self.selected_section = "new_profile".to_string();
            }
            if ui.button("PROFILES MANAGER").clicked() {
                self.selected_section = "profiles_manager".to_string();
            }
            if ui.button("PROXIES MANAGER").clicked() {
                self.selected_section = "proxies_manager".to_string();
            }

            ui.separator();
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Show the selected section based on the button clicked
            match self.selected_section.as_str() {
                "new_profile" => gui::new_profile_section::create_new_profile_section(ui, self),
                "profiles_manager" => { gui::single_profile_section::single_profile_section(ui, self); gui::profile_list_section::profile_list_section(ui, self); gui::active_profiles_section::active_profiles_section(ui, self); },
                "proxies_manager" => { gui::proxy_manager_section::proxy_manager_section(ui, self); gui::saved_proxies_section::saved_proxies_section(ui, self) },
                _ => {
                    ui.label("Select a section from the side panel.");
                },
            }

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
