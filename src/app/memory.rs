#[cfg(feature = "debug")]
pub mod memory {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use sysinfo::{Pid, ProcessesToUpdate, System};

    pub(crate) struct MemorySampler {
        system: System,
        pid: Pid,
        next_periodic_log: Instant,
        peak_rss: u64,
        peak_virtual: u64,
    }

    impl MemorySampler {
        pub(crate) fn from_feat() -> Option<Self> {
            Some(Self {
                system: System::new(),
                pid: Pid::from_u32(std::process::id()),
                next_periodic_log: Instant::now(),
                peak_rss: 0,
                peak_virtual: 0,
            })
        }

        pub(crate) fn from_env() -> Option<Self> {
            let enabled = std::env::var("PROTEUS_MEM_TRACE")
                .map(|value| {
                    let normalized = value.to_ascii_lowercase();
                    matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
                })
                .unwrap_or(false);

            if !enabled {
                return None;
            }

            Some(Self {
                system: System::new(),
                pid: Pid::from_u32(std::process::id()),
                next_periodic_log: Instant::now(),
                peak_rss: 0,
                peak_virtual: 0,
            })
        }

        pub(crate) fn maybe_log_periodic(&mut self, windows: usize, loaded_players: usize) {
            let now = Instant::now();
            if now < self.next_periodic_log {
                return;
            }
            self.next_periodic_log = now + Duration::from_secs(1);
            self.log("tick", windows, loaded_players);
        }

        pub(crate) fn log_event(&mut self, reason: &str, windows: usize, loaded_players: usize) {
            self.log(reason, windows, loaded_players);
        }

        fn log(&mut self, reason: &str, windows: usize, loaded_players: usize) {
            self.system
                .refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);

            let Some(process) = self.system.process(self.pid) else {
                return;
            };

            let rss = process.memory();
            let virtual_memory = process.virtual_memory();
            let thread_count = thread_count_for_pid(self.pid.as_u32());
            self.peak_rss = self.peak_rss.max(rss);
            self.peak_virtual = self.peak_virtual.max(virtual_memory);

            eprintln!(
                "[mem] reason={reason} rss_mb={:.1} virtual_mb={:.1} peak_rss_mb={:.1} peak_virtual_mb={:.1} threads={} windows={} loaded_players={}",
                bytes_to_mb(rss),
                bytes_to_mb(virtual_memory),
                bytes_to_mb(self.peak_rss),
                bytes_to_mb(self.peak_virtual),
                thread_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_owned()),
                windows,
                loaded_players,
            );
        }
    }

    fn bytes_to_mb(bytes: u64) -> f64 {
        bytes as f64 / (1024.0 * 1024.0)
    }

    #[cfg(target_os = "macos")]
    fn thread_count_for_pid(pid: u32) -> Option<usize> {
        let output = Command::new("ps")
            .args(["-M", "-p", &pid.to_string()])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8(output.stdout).ok()?;
        let line_count = stdout.lines().count();
        line_count.checked_sub(1)
    }

    #[cfg(target_os = "linux")]
    fn thread_count_for_pid(pid: u32) -> Option<usize> {
        let output = Command::new("ps")
            .args(["-L", "-p", &pid.to_string()])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8(output.stdout).ok()?;
        let line_count = stdout.lines().count();
        line_count.checked_sub(1)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn thread_count_for_pid(_pid: u32) -> Option<usize> {
        None
    }
}

#[cfg(not(feature = "debug"))]
mod memory {
    pub(crate) struct MemorySampler;

    impl MemorySampler {
        pub(crate) fn from_feat() -> Option<Self> {
            None
        }

        pub(crate) fn maybe_log_periodic(&mut self, _windows: usize, _loaded_players: usize) {}

        pub(crate) fn log_event(&mut self, _reason: &str, _windows: usize, _loaded_players: usize) {
        }
    }
}

pub(crate) use memory::MemorySampler;
