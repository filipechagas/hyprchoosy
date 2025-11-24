//! Logging initialization for debug builds.
//!
//! This module sets up file-based logging when the `debug` feature is enabled.

use anyhow::Result;

#[cfg(feature = "debug")]
use anyhow::Context;

#[cfg(feature = "debug")]
use log::info;
#[cfg(feature = "debug")]
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
#[cfg(feature = "debug")]
use std::env;
#[cfg(feature = "debug")]
use std::fs::File;

#[cfg(feature = "debug")]
pub fn init_logger() -> Result<()> {
    let log_dir = env::temp_dir().join("hyprchoosy");
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory at {}", log_dir.display()))?;

    let log_file = log_dir.join("hyprchoosy.log");

    let config = ConfigBuilder::new().set_time_format_rfc3339().build();

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        config,
        File::create(&log_file)
            .with_context(|| format!("Failed to create log file at {}", log_file.display()))?,
    )])
    .with_context(|| "Failed to initialize logger")?;

    info!("=== Hyprchoosy Debug Session Started ===");
    info!("Log file: {}", log_file.display());
    Ok(())
}

#[cfg(not(feature = "debug"))]
pub fn init_logger() -> Result<()> {
    Ok(())
}
