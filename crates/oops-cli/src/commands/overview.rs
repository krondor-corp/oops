//! Overview command -- default view showing volume summary + directory breakdown

use std::path::PathBuf;

use clap::Args;

use oops_core::{list_volumes, scan_top_entries, ScanOptions};

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Show disk usage overview (volumes + directory breakdown)
#[derive(Args, Debug, Clone)]
pub struct Overview {
    /// Target path to analyze
    pub path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum OverviewError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Overview {
    type Error = OverviewError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);
        let explicit = self.path.is_some() || ctx.explicit_path;

        let _ = explicit; // reserved for future use

        let opts = ScanOptions::default();
        let spinner = ui::Spinner::start("scanning...");
        let mut entries = scan_top_entries(target, &opts)?;
        spinner.stop();
        entries.sort_by(|a, b| b.size.cmp(&a.size));

        let total_size: u64 = entries.iter().map(|e| e.size).sum();

        // Find disk total for the volume containing the target path
        let disk_total = std::fs::canonicalize(target).ok().and_then(|canonical| {
            list_volumes().ok().and_then(|volumes| {
                volumes
                    .iter()
                    .filter(|v| canonical.starts_with(&v.mount_point))
                    .max_by_key(|v| v.mount_point.as_os_str().len())
                    .map(|v| v.total)
            })
        });

        ui::render_dir_breakdown(target, &entries, total_size, disk_total);

        Ok(NoOutput)
    }
}
