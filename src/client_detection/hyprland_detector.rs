use super::ClientDetector;
use serde::Deserialize;
use std::process::Command;

#[cfg(feature = "debug")]
use log::{debug, info};

#[derive(Deserialize)]
struct HyprlandWindow {
    class: String,
}

pub struct HyprlandDetector;

impl ClientDetector for HyprlandDetector {
    fn detect(&self) -> Option<String> {
        #[cfg(feature = "debug")]
        debug!("Attempting to detect client from Hyprland active window...");

        let output = Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
            .ok()?;

        if !output.status.success() {
            #[cfg(feature = "debug")]
            debug!("hyprctl command failed");
            return None;
        }

        let window: HyprlandWindow = serde_json::from_slice(&output.stdout).ok()?;
        let class = window.class.to_lowercase();

        if !class.is_empty() && class != "unknown" {
            #[cfg(feature = "debug")]
            info!("Detected client from Hyprland window: '{}'", class);
            return Some(class);
        }

        #[cfg(feature = "debug")]
        debug!("Could not extract valid class from Hyprland window");
        None
    }
}
