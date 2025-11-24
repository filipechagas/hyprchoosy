//! Client detection strategies.
//!
//! This module implements multiple strategies for detecting which application
//! initiated a URL open request, using a chain-of-responsibility pattern.

mod env_detector;
mod hyprland_detector;
mod process_tree_detector;

pub use env_detector::EnvDetector;
pub use hyprland_detector::HyprlandDetector;
pub use process_tree_detector::ProcessTreeDetector;

#[cfg(feature = "debug")]
use log::{info, warn};

pub trait ClientDetector {
    fn detect(&self) -> Option<String>;
}

pub fn detect_client() -> Option<String> {
    #[cfg(feature = "debug")]
    info!("Starting client detection...");

    let detectors: Vec<Box<dyn ClientDetector>> = vec![
        Box::new(HyprlandDetector),
        Box::new(EnvDetector),
        Box::new(ProcessTreeDetector),
    ];

    for detector in detectors {
        if let Some(client) = detector.detect() {
            return Some(client);
        }
    }

    #[cfg(feature = "debug")]
    warn!("All client detection methods failed");
    None
}
