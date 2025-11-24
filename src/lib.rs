//! Hyprchoosy - Smart browser chooser for Hyprland.
//!
//! This library provides functionality to route URLs to different browsers based on
//! the originating application and URL patterns.

pub mod client_detection;
pub mod config;
pub mod launcher;
pub mod logger;
pub mod matcher;

pub use client_detection::detect_client;
pub use config::{load_config, Config, DefaultSection, RuleSection};
pub use launcher::launch_browser;
pub use logger::init_logger;
pub use matcher::{match_client, match_host, parse_url_host};
