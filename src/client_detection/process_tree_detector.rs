use super::ClientDetector;
use sysinfo::{Pid, System};

#[cfg(feature = "debug")]
use log::{debug, info, warn};

const SKIP_LIST: &[&str] = &[
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

const MAX_STEPS: usize = 16;

pub struct ProcessTreeDetector;

impl ClientDetector for ProcessTreeDetector {
    fn detect(&self) -> Option<String> {
        #[cfg(feature = "debug")]
        debug!("Attempting to detect client from process tree...");

        let mut sys = System::new_all();
        sys.refresh_processes();

        let mut pid = Pid::from_u32(std::process::id());
        let mut steps = 0;

        #[cfg(feature = "debug")]
        debug!("Current PID: {}", pid);

        while steps < MAX_STEPS {
            let proc = sys.process(pid)?;
            let ppid = proc.parent()?;
            let parent = sys.process(ppid)?;

            let name = parent.name().to_lowercase();

            #[cfg(feature = "debug")]
            debug!(
                "Step {}: PID {} -> PPID {} (name: '{}')",
                steps, pid, ppid, name
            );

            let is_skipped = SKIP_LIST.iter().any(|s| name.contains(s));

            #[cfg(feature = "debug")]
            debug!(
                "  Name '{}' is {} wrapper",
                name,
                if is_skipped { "a" } else { "NOT a" }
            );

            if !is_skipped && !name.is_empty() {
                #[cfg(feature = "debug")]
                info!("Detected client from process tree: '{}'", name);
                return Some(name);
            }

            pid = ppid;
            steps += 1;
        }

        #[cfg(feature = "debug")]
        warn!(
            "Client detection from process tree failed after {} steps",
            steps
        );
        None
    }
}
