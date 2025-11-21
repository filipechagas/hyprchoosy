use anyhow::{bail, Result};
use hyprchoosy::{detect_client, launch, load_config, match_client, match_host, url_host};
use std::env;

fn main() -> Result<()> {
    let url = env::args().nth(1).unwrap_or_default();
    if url.is_empty() {
        bail!("Usage: hyprchoosy <URL>");
    }

    let cfg = load_config()?;
    let host = url_host(&url)?;
    let client = detect_client();

    // Priority: client -> host -> default
    if let Some(c) = client.as_deref() {
        if let Some(sec) = match_client(c, &cfg.sections) {
            return launch(&sec.browser, &url);
        }
    }

    if let Some(sec) = match_host(&host, &cfg.sections) {
        return launch(&sec.browser, &url);
    }

    launch(&cfg.default.browser, &url)
}
