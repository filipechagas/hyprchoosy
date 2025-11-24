//! Browser launching functionality.
//!
//! This module handles spawning browser processes in a detached manner
//! so they continue running after hyprchoosy exits.

use anyhow::{Context, Result};
use std::process::Command;

#[cfg(feature = "debug")]
use log::{info, warn};

pub fn launch_browser(browser: &str, url: &str) -> Result<()> {
    #[cfg(feature = "debug")]
    info!("Launching browser: '{}' with URL: '{}'", browser, url);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let mut cmd = Command::new(browser);
        cmd.arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        match cmd.spawn() {
            Ok(_) => {
                #[cfg(feature = "debug")]
                info!("Successfully spawned browser '{}'", browser);
                Ok(())
            }
            Err(e) => {
                #[cfg(feature = "debug")]
                warn!("Failed to spawn browser '{}': {}", browser, e);
                Err(e).with_context(|| format!("Failed to spawn browser '{}'", browser))
            }
        }
    }
    #[cfg(not(unix))]
    {
        match Command::new(browser).arg(url).spawn() {
            Ok(_) => {
                #[cfg(feature = "debug")]
                info!("Successfully spawned browser '{}'", browser);
                Ok(())
            }
            Err(e) => {
                #[cfg(feature = "debug")]
                warn!("Failed to spawn browser '{}': {}", browser, e);
                Err(e).with_context(|| format!("Failed to spawn browser '{}'", browser))
            }
        }
    }
}
