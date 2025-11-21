use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use sysinfo::{Pid, System};
use url::Url;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    default: DefaultSection,
    #[serde(flatten)]
    sections: HashMap<String, RuleSection>,
}

#[derive(Debug, Deserialize, Default)]
struct DefaultSection {
    #[serde(default = "default_browser")]
    browser: String,
}

fn default_browser() -> String {
    "firefox".to_string()
}

#[derive(Debug, Deserialize, Clone)]
struct RuleSection {
    browser: String,
    #[serde(default)]
    clients: Vec<String>,
    #[serde(default)]
    url: Vec<String>, // hostnames
}

fn xdg_config_home() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config")
}

fn config_path() -> PathBuf {
    // 1) HYPRCHOOSY_CONFIG overrides
    if let Ok(p) = env::var("HYPRCHOOSY_CONFIG") {
        return PathBuf::from(p);
    }
    // 2) default: $XDG_CONFIG_HOME/hyprchoosy/config.toml (fallback ~/.config)
    xdg_config_home().join("hyprchoosy/config.toml")
}

fn load_config() -> Result<Config> {
    let path = config_path();
    let data = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read config at {}. Set HYPRCHOOSY_CONFIG to override.",
            path.display()
        )
    })?;
    let cfg: Config = toml::from_str(&data).with_context(|| "Invalid TOML in config")?;
    Ok(cfg)
}

fn url_host(u: &str) -> Result<String> {
    // Accept schemeless inputs by prefixing http:// for parsing
    let parsed = if u.contains("://") {
        Url::parse(u)
    } else {
        Url::parse(&format!("http://{}", u))
    }
    .with_context(|| format!("Invalid URL: {}", u))?;
    Ok(parsed
        .host_str()
        .map(|s| s.to_lowercase())
        .unwrap_or_default())
}

// Walk up parent processes to find the originating client command name
fn detect_client() -> Option<String> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut pid = Pid::from_u32(std::process::id());
    let mut steps = 0usize;

    while steps < 16 {
        let proc = sys.process(pid)?;
        let ppid = proc.parent()?;
        let parent = sys.process(ppid)?;

        let name = parent.name().to_lowercase();

        // Skip common wrappers
        let skip = [
            "xdg-open",
            "gio",
            "systemd",
            "dbus-daemon",
            "bash",
            "sh",
            "zsh",
            "fish",
            "coreutils",
            "xdg-desktop-portal",
            "xdg-desktop-portal-gtk",
            "xdg-desktop-portal-hyprland",
        ];
        if !skip.iter().any(|s| name.contains(s)) && !name.is_empty() {
            return Some(name);
        }

        pid = ppid;
        steps += 1;
    }
    None
}

fn match_client<'a>(
    client: &str,
    sections: &'a HashMap<String, RuleSection>,
) -> Option<&'a RuleSection> {
    let c = client.to_lowercase();
    sections.values().find(|sec| {
        sec.clients
            .iter()
            .any(|needle| c.contains(&needle.to_lowercase()))
    })
}

fn match_host<'a>(
    host: &str,
    sections: &'a HashMap<String, RuleSection>,
) -> Option<&'a RuleSection> {
    let h = host.to_lowercase();
    sections.values().find(|sec| {
        sec.url.iter().any(|pat| {
            let p = pat.to_lowercase();
            h == p || h.ends_with(&format!(".{}", p))
        })
    })
}

fn launch(browser: &str, url: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let mut cmd = Command::new(browser);
        cmd.arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        unsafe {
            // Detach: new session
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        cmd.spawn()
            .with_context(|| format!("Failed to spawn browser '{}'", browser))?;
    }
    #[cfg(not(unix))]
    {
        Command::new(browser)
            .arg(url)
            .spawn()
            .with_context(|| format!("Failed to spawn browser '{}'", browser))?;
    }
    Ok(())
}

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
