use std::path::PathBuf;
use std::process::Command;

use tracing::debug;

#[derive(Debug, Clone)]
pub struct Volume {
    pub filesystem: String,
    pub mount_point: PathBuf,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub capacity_pct: f64,
}

impl Volume {
    pub fn free(&self) -> u64 {
        self.available
    }
}

/// List mounted volumes by parsing `df -Pk` (POSIX mode, 1K blocks).
///
/// POSIX output: Filesystem 1024-blocks Used Available Capacity Mounted-on
pub fn list_volumes() -> Result<Vec<Volume>, crate::Error> {
    debug!("running df -Pk");
    let output = Command::new("df")
        .args(["-Pk"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut volumes = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let filesystem = parts[0].to_string();

        if filesystem == "devfs"
            || filesystem.starts_with("map")
            || filesystem == "none"
        {
            continue;
        }

        let total = parts[1].parse::<u64>().unwrap_or(0) * 1024;
        let used = parts[2].parse::<u64>().unwrap_or(0) * 1024;
        let available = parts[3].parse::<u64>().unwrap_or(0) * 1024;
        let capacity_str = parts[4].trim_end_matches('%');
        let capacity_pct = capacity_str.parse::<f64>().unwrap_or(0.0);
        let mount_point = PathBuf::from(parts[5..].join(" "));

        if total == 0 {
            continue;
        }

        volumes.push(Volume {
            filesystem,
            mount_point,
            total,
            used,
            available,
            capacity_pct,
        });
    }

    volumes.dedup_by(|a, b| a.mount_point == b.mount_point);
    debug!(count = volumes.len(), "found volumes");
    Ok(volumes)
}
