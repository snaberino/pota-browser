use eframe::egui;

use crate::chromium::chromium::ChromiumProfile;

/// Renders a dropdown menu for WebRTC configuration and updates the profile if changed.
pub fn render_webrtc_dropdown( ui: &mut egui::Ui, current_webrtc: &mut String, profile_name: &str, profiles: &mut Vec<ChromiumProfile>, log_message: &mut String ) {
    let old_webrtc = current_webrtc.clone();
    // ui.label("WebRTC");
    egui::ComboBox::from_id_salt(format!("webrtc_{}", profile_name))
        .selected_text(&*current_webrtc)
        .show_ui(ui, |ui| {
            ui.selectable_value(current_webrtc, "default".to_string(), "default");
            ui.selectable_value(current_webrtc, "fake".to_string(), "fake");
            ui.selectable_value(current_webrtc, "block".to_string(), "block");
        });

    // Update only if changed
    if old_webrtc != *current_webrtc {
        if let Some(profile) = profiles.iter_mut().find(|p| p.name == profile_name) {
            profile.webrtc = current_webrtc.clone();
            ChromiumProfile::save_profile_configs(profiles);
            *log_message = format!(
                "WebRTC spoofing for profile {} set to {}.",
                profile_name, current_webrtc
            );
        }
    }
}

/// Renders a checkbox for enabling or disabling the debugging port and updates the profile if changed.
pub fn render_debug_checkbox( ui: &mut egui::Ui, profile: &mut ChromiumProfile, profiles: &mut Vec<ChromiumProfile>, log_message: &mut String) {
    let mut debug_enabled = profile.debugging_port != 0;

    if ui.checkbox(&mut debug_enabled, format!("{}", profile.debugging_port)).changed() {
        if debug_enabled {
            // Assign a unique port based on the profile index
            let base_port = 9222;
            let profile_index = profiles
                .iter()
                .position(|p| p.name == profile.name)
                .unwrap_or(0);
            profile.debugging_port = base_port + profile_index as u16;
        } else {
            profile.debugging_port = 0;
        }

        // Update the profile in the profiles list
        if let Some(p) = profiles.iter_mut().find(|p| p.name == profile.name) {
            p.debugging_port = profile.debugging_port;
        }

        ChromiumProfile::save_profile_configs(profiles);
        *log_message = format!(
            "Debugging port for profile {} set to {}.",
            profile.name, profile.debugging_port
        );
    }
}

/// Renders a checkbox for enabling or disabling headless mode and updates the profile if changed.
pub fn render_headless_checkbox( ui: &mut egui::Ui, profile: &mut ChromiumProfile, profiles: &mut Vec<ChromiumProfile>, log_message: &mut String ) {
    let mut headless_enabled = profile.headless;

    if ui.toggle_value(&mut headless_enabled, format!("{}", profile.headless)).changed() {
    // if ui.checkbox(&mut headless_enabled, format!("{}", profile.headless)).changed() {
        profile.headless = headless_enabled;

        // Update the profile in the profiles list
        if let Some(p) = profiles.iter_mut().find(|p| p.name == profile.name) {
            p.headless = profile.headless;
        }

        ChromiumProfile::save_profile_configs(profiles);
        *log_message = format!(
            "Headless mode for profile {} set to {}.",
            profile.name, profile.headless
        );
    }
}