mod gui;
mod chromium;
mod proxy;

use eframe::{egui, Frame};
use eframe::egui::Context;

use crate::gui::custom_browser::CustomBrowserPanel;
use crate::gui::logger::LoggerPanel;

use crate::gui::new_profile::NewProfilePanel;
use crate::chromium::chromium_manager::ChromiumManager;
use crate::gui::profiles_list::ProfilesListPanel;

use crate::gui::new_proxy::NewProxyPanel;
use crate::proxy::proxy_manager::ProxyManager;
use crate::gui::proxies_list::ProxiesListPanel;

enum View {
    CustomBrowsersPanel,
    ProfilesListPanel,
    ProxiesPanel,
}

struct PotaBrowser {
    current_view: View,
    logger_panel: LoggerPanel,
    custom_browsers_panel: CustomBrowserPanel,
    new_profile_panel: NewProfilePanel,

    profiles_list_panel: ProfilesListPanel,
    profiles: ChromiumManager,

    new_proxy_panel: NewProxyPanel,
    proxy_manager: ProxyManager,
    proxies_list_panel: ProxiesListPanel,
}

impl PotaBrowser {
    fn default() -> Self {
        Self {
            current_view: View::CustomBrowsersPanel,
            logger_panel: LoggerPanel::new(),
            custom_browsers_panel: CustomBrowserPanel::new(),
            new_profile_panel: NewProfilePanel::new(),

            profiles_list_panel: ProfilesListPanel::new(),
            profiles: ChromiumManager::default(),

            new_proxy_panel: NewProxyPanel::new(),
            proxy_manager: ProxyManager::default(),
            proxies_list_panel: ProxiesListPanel::new(),
        }
    }
}

impl eframe::App for PotaBrowser {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Custom Browsers").clicked() {
                    self.current_view = View::CustomBrowsersPanel;
                }
                if ui.button("Profiles Manager").clicked() {
                    self.current_view = View::ProfilesListPanel;
                }
                if ui.button("Proxies Manager").clicked() {
                    self.current_view = View::ProxiesPanel;
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::CustomBrowsersPanel => self.custom_browsers_panel.ui(ui, &mut self.logger_panel),
                // View::NewProfilePanel => self.new_profile_panel.ui(ui, &mut self.logger_panel, &mut self.custom_browsers_panel, &mut self.profiles),
                View::ProfilesListPanel => {
                    self.new_profile_panel.ui(ui, &mut self.logger_panel, &mut self.custom_browsers_panel, &mut self.profiles);
                    ui.separator();
                    self.profiles_list_panel.ui(ui, &mut self.logger_panel, &mut self.profiles, &self.proxy_manager);
                },
                View::ProxiesPanel => {
                    self.new_proxy_panel.ui(ui, &mut self.logger_panel, &mut self.proxy_manager);
                    ui.separator();
                    self.proxies_list_panel.ui(ui, &mut self.logger_panel, &mut self.proxy_manager)
                },
            }
        });
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            self.logger_panel.ui(ui);
        });
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions::default();
    options.viewport.resizable = Some(true);
    options.centered = true;
    eframe::run_native(
        "pota browser",
        options,
        Box::new(|_cc| Ok(Box::new(PotaBrowser::default()))),
    )
}
