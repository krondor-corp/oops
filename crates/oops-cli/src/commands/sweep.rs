//! Sweep command -- find common disk space wasters

use std::path::PathBuf;

use clap::Args;

use oops_core::sweep_directory;

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Find common disk space wasters (node_modules, caches, build artifacts)
#[derive(Args, Debug, Clone)]
pub struct Sweep {
    /// Target path to scan
    pub path: Option<PathBuf>,

    /// Maximum depth to scan for waste
    #[arg(short, long, default_value = "6")]
    pub depth: usize,

    /// Show individual entries (not just summaries)
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SweepError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Sweep {
    type Error = SweepError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);

        let spinner = ui::Spinner::start(&format!("sweeping {} for reclaimable space...", ui::short_path(target)));
        let entries = sweep_directory(target, self.depth)?;
        spinner.stop();
        ui::render_sweep_results(&entries, self.verbose);
        Ok(NoOutput)
    }
}
