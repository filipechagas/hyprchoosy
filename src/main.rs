use anyhow::{bail, Result};
use hyprchoosy::{
    detect_client, init_logger, launch_browser, load_config, match_client, match_host,
    parse_url_host,
};
use std::env;

#[cfg(feature = "debug")]
use log::info;

fn main() -> Result<()> {
    init_logger()?;

    #[cfg(feature = "debug")]
    info!("=== Starting hyprchoosy ===");

    let url = env::args().nth(1).unwrap_or_default();

    #[cfg(feature = "debug")]
    info!("Received URL: '{}'", url);

    if url.is_empty() {
        #[cfg(feature = "debug")]
        log::error!("No URL provided");
        bail!("Usage: hyprchoosy <URL>");
    }

    #[cfg(feature = "debug")]
    info!("Loading configuration...");
    let cfg = load_config()?;

    #[cfg(feature = "debug")]
    info!("Configuration loaded successfully");

    let host = parse_url_host(&url)?;
    #[cfg(feature = "debug")]
    info!("Extracted host: '{}'", host);

    let client = detect_client();
    #[cfg(feature = "debug")]
    info!("Detected client: {:?}", client);

    if let Some(c) = client.as_deref() {
        #[cfg(feature = "debug")]
        info!("Checking client rules for '{}'", c);

        if let Some(sec) = match_client(c, &cfg.sections) {
            #[cfg(feature = "debug")]
            info!("Using browser from client rule: '{}'", sec.browser);
            return launch_browser(&sec.browser, &url);
        }
    } else {
        #[cfg(feature = "debug")]
        info!("No client detected, skipping client rules");
    }

    #[cfg(feature = "debug")]
    info!("Checking host rules for '{}'", host);

    if let Some(sec) = match_host(&host, &cfg.sections) {
        #[cfg(feature = "debug")]
        info!("Using browser from host rule: '{}'", sec.browser);
        return launch_browser(&sec.browser, &url);
    }

    #[cfg(feature = "debug")]
    info!(
        "No rules matched, using default browser: '{}'",
        cfg.default.browser
    );
    launch_browser(&cfg.default.browser, &url)
}
