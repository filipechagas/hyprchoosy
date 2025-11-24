use super::ClientDetector;
use std::env;

#[cfg(feature = "debug")]
use log::{debug, info};

pub struct EnvDetector;

impl ClientDetector for EnvDetector {
    fn detect(&self) -> Option<String> {
        if let Ok(desktop_file) = env::var("GIO_LAUNCHED_DESKTOP_FILE") {
            #[cfg(feature = "debug")]
            debug!("Found GIO_LAUNCHED_DESKTOP_FILE: {}", desktop_file);

            if let Some(filename) = desktop_file.rsplit('/').next() {
                let app_name = filename.trim_end_matches(".desktop");

                if app_name == "hyprchoosy" {
                    #[cfg(feature = "debug")]
                    debug!("Skipping hyprchoosy.desktop (looking for originating app)");
                    return None;
                }

                #[cfg(feature = "debug")]
                info!("Detected client from env: '{}'", app_name);
                return Some(app_name.to_string());
            }
        }

        None
    }
}
