//! Config command -- inspect and initialize oops configuration

use clap::{Args, Subcommand};

use oops_core::config::{self, OopsConfig};
use oops_core::resolve_rules;

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Manage oops configuration
#[derive(Args, Debug, Clone)]
pub struct Config {
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigAction {
    /// Create default config file
    Init,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Config {
    type Error = ConfigError;
    type Output = NoOutput;

    fn run(&self, _ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        match &self.action {
            None => show_config(),
            Some(ConfigAction::Init) => init_config(),
        }
    }
}

fn show_config() -> Result<NoOutput, ConfigError> {
    let path = config::config_path();

    if !path.exists() {
        let dir = config::config_dir();
        std::fs::create_dir_all(&dir).map_err(oops_core::Error::Io)?;
        std::fs::write(&path, config::DEFAULT_CONFIG_CONTENT).map_err(oops_core::Error::Io)?;
        ui::success(&format!("created {}", path.display()));
    }

    ui::header("oops config");
    eprintln!();
    eprintln!(
        "  {} {}",
        ui::bold("path:"),
        ui::highlight(&path.display().to_string()),
    );

    let config = OopsConfig::load()?;
    let rules = resolve_rules(&config)?;

    let builtin_count = oops_core::builtin_rules().len();
    let disabled = config.disabled_builtins();
    let user_added = config
        .rules
        .iter()
        .filter(|r| r.enabled != Some(false))
        .filter(|r| !oops_core::builtin_rules().iter().any(|b| b.name == r.name))
        .count();

    eprintln!();
    ui::header("rules");
    eprintln!();
    eprintln!(
        "  {} {} built-in, {} active",
        ui::bold("loaded:"),
        builtin_count,
        rules.len(),
    );

    if !disabled.is_empty() {
        eprintln!("  {} {}", ui::bold("disabled:"), disabled.join(", "),);
    }

    if user_added > 0 {
        eprintln!("  {} {}", ui::bold("user-added:"), user_added,);
    }

    eprintln!();
    for rule in &rules {
        let clean_indicator = if rule.clean.is_some() { "+" } else { "-" };
        eprintln!(
            "  {} {} {}  {}",
            ui::dim(clean_indicator),
            ui::highlight(&rule.name),
            ui::dim(&format!("({})", rule.description)),
            match &rule.match_mode {
                oops_core::MatchMode::DirBasename(d) => format!("dir={d}"),
                oops_core::MatchMode::AbsolutePath(p) => format!("path={}", ui::short_path(p)),
                oops_core::MatchMode::FileGlob(g) => format!("glob={g}"),
            },
        );
    }

    Ok(NoOutput)
}

fn init_config() -> Result<NoOutput, ConfigError> {
    let path = config::config_path();

    if path.exists() {
        ui::detail(&format!("config already exists at {}", path.display()));
    } else {
        let dir = config::config_dir();
        std::fs::create_dir_all(&dir).map_err(oops_core::Error::Io)?;
        std::fs::write(&path, config::DEFAULT_CONFIG_CONTENT).map_err(oops_core::Error::Io)?;
        ui::success(&format!("created {}", path.display()));
    }
    Ok(NoOutput)
}
