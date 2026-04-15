//! Tree command -- visual directory size tree

use std::path::PathBuf;

use clap::Args;

use oops_core::{scan_top_entries, ScanOptions};

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Show a visual directory size tree
#[derive(Args, Debug, Clone)]
pub struct Tree {
    /// Target path to analyze
    pub path: Option<PathBuf>,

    /// Maximum depth to display
    #[arg(short, long, default_value = "3")]
    pub depth: usize,

    /// Minimum percentage of parent to show
    #[arg(long, default_value = "1.0")]
    pub min_pct: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum TreeError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Tree {
    type Error = TreeError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);
        let opts = ScanOptions::default();

        let spinner = ui::Spinner::start("scanning...");
        let entries = scan_top_entries(target, &opts)?;
        spinner.stop();
        let total: u64 = entries.iter().map(|e| e.size).sum();

        ui::header(&ui::short_path(target).to_string());
        eprintln!("  {} total", ui::highlight(&ui::fmt_size(total)));
        eprintln!();

        self.print_level(target, &opts, 0)?;
        Ok(NoOutput)
    }
}

impl Tree {
    fn print_level(
        &self,
        path: &std::path::Path,
        opts: &ScanOptions,
        depth: usize,
    ) -> Result<(), TreeError> {
        if depth >= self.depth {
            return Ok(());
        }

        let mut entries = scan_top_entries(path, opts)?;
        entries.sort_by(|a, b| b.size.cmp(&a.size));

        let parent_total: u64 = entries.iter().map(|e| e.size).sum();

        ui::render_tree_node(&entries, parent_total, depth, self.min_pct);

        // Recurse into directories that passed the filter
        for entry in &entries {
            if entry.is_dir && parent_total > 0 {
                let pct = (entry.size as f64 / parent_total as f64) * 100.0;
                if pct >= self.min_pct {
                    self.print_level(&entry.path, opts, depth + 1)?;
                }
            }
        }

        Ok(())
    }
}
