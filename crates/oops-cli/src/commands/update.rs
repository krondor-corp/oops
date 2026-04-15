//! Update command -- update oops to the latest release.

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::Args;

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

const GITHUB_REPO: &str = "krondor-corp/oops";
const INSTALL_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/krondor-corp/oops/main/install.sh";

/// Update oops to the latest release
#[derive(Args, Debug, Clone)]
pub struct Update {
    /// Force reinstall even when already up to date
    #[arg(long, short)]
    pub force: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("{0}")]
    Failed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InstallMethod {
    Script(PathBuf),
    Cargo(PathBuf),
    Source(PathBuf),
    Unknown(PathBuf),
}

impl InstallMethod {
    fn description(&self) -> &str {
        match self {
            InstallMethod::Script(_) => "install script (~/.local/bin)",
            InstallMethod::Cargo(_) => "cargo install (~/.cargo/bin)",
            InstallMethod::Source(_) => "source build (target/)",
            InstallMethod::Unknown(_) => "unknown",
        }
    }
}

impl Op for Update {
    type Error = UpdateError;
    type Output = NoOutput;

    fn run(&self, _ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let install_method = detect_installation()?;
        let current_version = env!("CARGO_PKG_VERSION");

        ui::header("Update");
        eprintln!("  Current version: {}", ui::highlight(current_version));
        eprintln!("  Installation: {}", ui::dim(install_method.description()));
        eprintln!();

        ui::progress("Checking for updates...");
        let latest_version = get_latest_version()?;
        eprintln!("  Latest version: {}", ui::highlight(&latest_version));
        eprintln!();

        let needs_update = is_newer_version(current_version, &latest_version);

        if !needs_update && !self.force {
            ui::success("Already up to date!");
            return Ok(NoOutput);
        }

        if needs_update {
            ui::progress(&format!(
                "New version available: {} -> {}",
                ui::dim(current_version),
                ui::highlight(&latest_version),
            ));
        } else {
            ui::progress("Forcing reinstall...");
        }

        match install_method {
            InstallMethod::Script(_) => {
                run_install_script()?;
            }
            InstallMethod::Cargo(_) | InstallMethod::Source(_) => {
                eprintln!();
                eprintln!("You're running a development build.");

                if prompt_confirm("Install latest release to ~/.local/bin?", true)? {
                    run_install_script()?;
                } else {
                    eprintln!();
                    eprintln!("To update manually, run:");
                    ui::detail(&format!(
                        "cargo install --git https://github.com/{}",
                        GITHUB_REPO,
                    ));
                    return Ok(NoOutput);
                }
            }
            InstallMethod::Unknown(ref path) => {
                eprintln!();
                eprintln!(
                    "Unknown installation method: {}",
                    ui::dim(&path.display().to_string()),
                );
                eprintln!();
                eprintln!("To install via script (recommended):");
                ui::detail(&format!("curl -fsSL {} | bash", INSTALL_SCRIPT_URL));
                eprintln!();
                eprintln!("Or rebuild from source:");
                ui::detail(&format!(
                    "cargo install --git https://github.com/{}",
                    GITHUB_REPO,
                ));
                return Ok(NoOutput);
            }
        }

        eprintln!();
        ui::success("Updated successfully!");

        Ok(NoOutput)
    }
}

fn detect_installation() -> Result<InstallMethod, UpdateError> {
    let exe_path = std::env::current_exe()?;
    let path_str = exe_path.to_string_lossy();

    if path_str.contains("/.local/bin/") {
        Ok(InstallMethod::Script(exe_path))
    } else if path_str.contains("/.cargo/bin/") {
        Ok(InstallMethod::Cargo(exe_path))
    } else if path_str.contains("/target/") {
        Ok(InstallMethod::Source(exe_path))
    } else {
        Ok(InstallMethod::Unknown(exe_path))
    }
}

fn get_latest_version() -> Result<String, UpdateError> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            &format!(
                "https://api.github.com/repos/{}/releases/latest",
                GITHUB_REPO,
            ),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UpdateError::Failed(format!(
            "Failed to fetch latest version: {}",
            stderr,
        )));
    }

    let body = String::from_utf8_lossy(&output.stdout);

    for line in body.lines() {
        if line.contains("\"tag_name\"") {
            if let Some(start) = line.find(':') {
                let value = &line[start + 1..];
                let value = value.trim().trim_end_matches(',').trim_matches('"');
                let version = value.trim_start_matches('v');
                return Ok(version.to_string());
            }
        }
    }

    Err(UpdateError::Failed(
        "Could not parse version from GitHub response".to_string(),
    ))
}

fn prompt_confirm(message: &str, default_yes: bool) -> Result<bool, UpdateError> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    eprint!("{} {} ", message, suffix);
    io::stderr().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    let answer = line.trim().to_lowercase();
    if answer.is_empty() {
        Ok(default_yes)
    } else {
        Ok(answer == "y" || answer == "yes")
    }
}

fn run_install_script() -> Result<(), UpdateError> {
    eprintln!();
    ui::progress("Installing via install script...");
    eprintln!();

    let status = Command::new("bash")
        .args(["-c", &format!("curl -fsSL {} | bash", INSTALL_SCRIPT_URL)])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(UpdateError::Failed("Install script failed".to_string()));
    }

    Ok(())
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v
            .trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        (
            *parts.first().unwrap_or(&0),
            *parts.get(1).unwrap_or(&0),
            *parts.get(2).unwrap_or(&0),
        )
    };

    let current = parse_version(current);
    let latest = parse_version(latest);

    latest > current
}
