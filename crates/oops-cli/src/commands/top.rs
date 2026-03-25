//! Top command -- find largest files and directories

use std::path::PathBuf;

use clap::Args;

use oops_core::{ScanOptions, scan_directory};

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Find the largest files and directories
#[derive(Args, Debug, Clone)]
pub struct Top {
    /// Target path to scan
    pub path: Option<PathBuf>,

    /// Number of results to show
    #[arg(short = 'n', long, default_value = "20")]
    pub count: usize,

    /// Maximum depth to scan
    #[arg(short, long, default_value = "5")]
    pub depth: usize,

    /// Show only files (no directories)
    #[arg(long)]
    pub files_only: bool,

    /// Show only directories
    #[arg(long)]
    pub dirs_only: bool,

    /// Minimum size to show (e.g. "100MB", "1GB")
    #[arg(long)]
    pub min_size: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TopError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
    #[error("invalid size: {0}")]
    InvalidSize(String),
}

impl Op for Top {
    type Error = TopError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);
        let min_bytes = self.parse_min_size()?;

        let opts = ScanOptions {
            max_depth: Some(self.depth),
            ..Default::default()
        };

        let spinner = ui::Spinner::start(&format!("scanning {} (depth {})...", ui::short_path(target), self.depth));
        let mut entries = scan_directory(target, &opts)?;
        spinner.stop();

        if self.files_only {
            entries.retain(|e| !e.is_dir);
        }
        if self.dirs_only {
            entries.retain(|e| e.is_dir);
        }
        if let Some(min) = min_bytes {
            entries.retain(|e| e.size >= min);
        }

        entries.sort_by(|a, b| b.size.cmp(&a.size));
        entries.truncate(self.count);

        ui::render_top_entries(&entries, target);
        Ok(NoOutput)
    }
}

impl Top {
    fn parse_min_size(&self) -> Result<Option<u64>, TopError> {
        let s = match &self.min_size {
            Some(s) => s,
            None => return Ok(None),
        };

        let s = s.trim().to_uppercase();
        let (num_str, multiplier) = if let Some(n) = s.strip_suffix("GB") {
            (n, 1024u64 * 1024 * 1024)
        } else if let Some(n) = s.strip_suffix("MB") {
            (n, 1024 * 1024)
        } else if let Some(n) = s.strip_suffix("KB") {
            (n, 1024)
        } else if let Some(n) = s.strip_suffix('B') {
            (n, 1)
        } else {
            (s.as_str(), 1)
        };

        let num: f64 = num_str.trim().parse().map_err(|_| TopError::InvalidSize(s.clone()))?;
        Ok(Some((num * multiplier as f64) as u64))
    }
}
