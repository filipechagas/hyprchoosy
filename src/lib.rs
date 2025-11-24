use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use sysinfo::{Pid, System};
use url::Url;

#[cfg(feature = "debug")]
use log::{debug, info, warn};
#[cfg(feature = "debug")]
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
#[cfg(feature = "debug")]
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultSection,
    #[serde(flatten)]
    pub sections: HashMap<String, RuleSection>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DefaultSection {
    #[serde(default = "default_browser")]
    pub browser: String,
}

pub fn default_browser() -> String {
    "firefox".to_string()
}

#[cfg(feature = "debug")]
pub fn init_logger() -> Result<()> {
    let log_dir = env::temp_dir().join("hyprchoosy");
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory at {}", log_dir.display()))?;
    
    let log_file = log_dir.join("hyprchoosy.log");
    
    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .build();
    
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
    // No-op when debug feature is not enabled
    Ok(())
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleSection {
    pub browser: String,
    #[serde(default)]
    pub clients: Vec<String>,
    #[serde(default)]
    pub url: Vec<String>, // hostnames
}

pub fn xdg_config_home() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config")
}

pub fn config_path() -> PathBuf {
    // 1) HYPRCHOOSY_CONFIG overrides
    if let Ok(p) = env::var("HYPRCHOOSY_CONFIG") {
        return PathBuf::from(p);
    }
    // 2) default: $XDG_CONFIG_HOME/hyprchoosy/config.toml (fallback ~/.config)
    xdg_config_home().join("hyprchoosy/config.toml")
}

pub fn load_config() -> Result<Config> {
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

pub fn url_host(u: &str) -> Result<String> {
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

// Detect client from environment variable (for systemd/DBus launched apps)
fn detect_from_env() -> Option<String> {
    // Check GIO_LAUNCHED_DESKTOP_FILE (set by gio/xdg-open)
    if let Ok(desktop_file) = env::var("GIO_LAUNCHED_DESKTOP_FILE") {
        #[cfg(feature = "debug")]
        debug!("Found GIO_LAUNCHED_DESKTOP_FILE: {}", desktop_file);
        
        // Extract app name from path like /usr/share/applications/thunderbird.desktop
        if let Some(filename) = desktop_file.rsplit('/').next() {
            let app_name = filename.trim_end_matches(".desktop");
            
            // Skip if it's hyprchoosy itself (we're looking for the originating app)
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

// Detect client from active Hyprland window
fn detect_from_hyprland() -> Option<String> {
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
    
    let json = String::from_utf8(output.stdout).ok()?;
    
    #[cfg(feature = "debug")]
    debug!("Hyprctl output: {}", json);
    
    // Simple JSON parsing to extract class field
    // Format: {"class": "thunderbird", ...}
    for line in json.lines() {
        if line.contains("\"class\"") {
            if let Some(class_value) = line.split(':').nth(1) {
                let class = class_value
                    .trim()
                    .trim_matches(',')
                    .trim_matches('"')
                    .to_lowercase();
                
                if !class.is_empty() && class != "unknown" {
                    #[cfg(feature = "debug")]
                    info!("Detected client from Hyprland window: '{}'", class);
                    return Some(class);
                }
            }
        }
    }
    
    #[cfg(feature = "debug")]
    debug!("Could not extract class from Hyprland window");
    None
}

// Walk up parent processes to find the originating client command name
fn detect_from_process_tree() -> Option<String> {
    #[cfg(feature = "debug")]
    debug!("Attempting to detect client from process tree...");
    
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut pid = Pid::from_u32(std::process::id());
    let mut steps = 0usize;
    
    #[cfg(feature = "debug")]
    debug!("Current PID: {}", pid);

    while steps < 16 {
        let proc = sys.process(pid);
        if proc.is_none() {
            #[cfg(feature = "debug")]
            warn!("Could not find process for PID: {}", pid);
            return None;
        }
        let proc = proc?;
        
        let ppid = proc.parent();
        if ppid.is_none() {
            #[cfg(feature = "debug")]
            warn!("Process {} has no parent", pid);
            return None;
        }
        let ppid = ppid?;
        
        let parent = sys.process(ppid);
        if parent.is_none() {
            #[cfg(feature = "debug")]
            warn!("Could not find parent process for PPID: {}", ppid);
            return None;
        }
        let parent = parent?;

        let name = parent.name().to_lowercase();
        
        #[cfg(feature = "debug")]
        debug!("Step {}: PID {} -> PPID {} (name: '{}')", steps, pid, ppid, name);

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
        
        let is_skipped = skip.iter().any(|s| name.contains(s));
        #[cfg(feature = "debug")]
        debug!("  Name '{}' is {} wrapper", name, if is_skipped { "a" } else { "NOT a" });
        
        if !is_skipped && !name.is_empty() {
            #[cfg(feature = "debug")]
            info!("Detected client from process tree: '{}'", name);
            return Some(name);
        }

        pid = ppid;
        steps += 1;
    }
    
    #[cfg(feature = "debug")]
    warn!("Client detection from process tree failed after {} steps", steps);
    None
}

// Try multiple methods to detect the originating client
pub fn detect_client() -> Option<String> {
    #[cfg(feature = "debug")]
    info!("Starting client detection...");
    
    // Method 1: Check active Hyprland window (most reliable for GUI apps)
    if let Some(client) = detect_from_hyprland() {
        return Some(client);
    }
    
    // Method 2: Check environment variables
    if let Some(client) = detect_from_env() {
        return Some(client);
    }
    
    // Method 3: Check process tree (fallback)
    if let Some(client) = detect_from_process_tree() {
        return Some(client);
    }
    
    #[cfg(feature = "debug")]
    warn!("All client detection methods failed");
    None
}

pub fn match_client<'a>(
    client: &str,
    sections: &'a HashMap<String, RuleSection>,
) -> Option<&'a RuleSection> {
    #[cfg(feature = "debug")]
    debug!("Matching client: '{}'", client);
    
    let c = client.to_lowercase();
    
    #[cfg(feature = "debug")]
    debug!("Available sections: {:?}", sections.keys().collect::<Vec<_>>());
    
    for (_section_name, sec) in sections.iter() {
        #[cfg(feature = "debug")]
        debug!("  Checking section '{}' with clients: {:?}", _section_name, sec.clients);
        
        for needle in &sec.clients {
            let needle_lower = needle.to_lowercase();
            if c.contains(&needle_lower) {
                #[cfg(feature = "debug")]
                info!("Client '{}' matched rule '{}' (pattern: '{}')", client, _section_name, needle);
                return Some(sec);
            }
        }
    }
    
    #[cfg(feature = "debug")]
    debug!("No client match found for '{}'", client);
    None
}

pub fn match_host<'a>(
    host: &str,
    sections: &'a HashMap<String, RuleSection>,
) -> Option<&'a RuleSection> {
    #[cfg(feature = "debug")]
    debug!("Matching host: '{}'", host);
    
    let h = host.to_lowercase();
    
    for (_section_name, sec) in sections.iter() {
        #[cfg(feature = "debug")]
        debug!("  Checking section '{}' with URL patterns: {:?}", _section_name, sec.url);
        
        for pat in &sec.url {
            let p = pat.to_lowercase();
            let matches = h == p || h.ends_with(&format!(".{}", p));
            
            #[cfg(feature = "debug")]
            debug!("    Pattern '{}' {} match host '{}'", pat, if matches { "DOES" } else { "does NOT" }, host);
            
            if matches {
                #[cfg(feature = "debug")]
                info!("Host '{}' matched rule '{}' (pattern: '{}')", host, _section_name, pat);
                return Some(sec);
            }
        }
    }
    
    #[cfg(feature = "debug")]
    debug!("No host match found for '{}'", host);
    None
}

pub fn launch(browser: &str, url: &str) -> Result<()> {
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
            // Detach: new session
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
