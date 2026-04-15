//! Drill command -- follow the biggest child at each level

use std::path::PathBuf;

use clap::Args;

use oops_core::{scan_top_entries, DirEntry, ScanOptions};

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Drill into the largest subdirectory at each level
#[derive(Args, Debug, Clone)]
pub struct Drill {
    /// Target path to drill into
    pub path: Option<PathBuf>,

    /// Maximum levels to drill down
    #[arg(short, long, default_value = "10")]
    pub depth: usize,

    /// Stop drilling when the largest child is below this percentage of its parent
    #[arg(long, default_value = "25.0")]
    pub threshold: f64,

    /// Number of sibling entries to show at each level
    #[arg(short = 'n', long, default_value = "5")]
    pub show: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum DrillError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

/// One level of the drill-down trace.
pub struct DrillLevel {
    pub dir_name: String,
    pub total: u64,
    pub entries: Vec<DirEntry>,
    pub show_count: usize,
    pub stop_reason: Option<StopReason>,
}

pub enum StopReason {
    BelowThreshold { pct: f64, threshold: f64 },
    NoSubdirectories,
    Empty,
}

impl Op for Drill {
    type Error = DrillError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let opts = ScanOptions::default();
        let start = self.path.as_ref().unwrap_or(&ctx.path);
        let mut current = start.clone();
        let mut trail: Vec<String> = Vec::new();

        for depth in 0..self.depth {
            let spinner = ui::Spinner::start("scanning...");
            let mut entries = scan_top_entries(&current, &opts)?;
            spinner.stop();
            entries.sort_by(|a, b| b.size.cmp(&a.size));

            let total: u64 = entries.iter().map(|e| e.size).sum();
            let dir_name = dir_display_name(&current, start);

            if total == 0 || entries.is_empty() {
                let level = DrillLevel {
                    dir_name: dir_name.clone(),
                    total,
                    entries,
                    show_count: 0,
                    stop_reason: Some(StopReason::Empty),
                };
                trail.push(dir_name);
                ui::render_drill_level(&level, depth, false);
                break;
            }

            // Find biggest dir and compute stop reason before moving entries
            let biggest_dir_path = entries.iter().find(|e| e.is_dir).map(|e| e.path.clone());
            let stop_reason = match &biggest_dir_path {
                Some(_) => {
                    let biggest = entries.iter().find(|e| e.is_dir).unwrap();
                    let pct = (biggest.size as f64 / total as f64) * 100.0;
                    if pct < self.threshold {
                        Some(StopReason::BelowThreshold {
                            pct,
                            threshold: self.threshold,
                        })
                    } else {
                        None
                    }
                }
                None => Some(StopReason::NoSubdirectories),
            };

            let should_stop = stop_reason.is_some();

            let level = DrillLevel {
                dir_name: dir_name.clone(),
                total,
                entries,
                show_count: self.show,
                stop_reason,
            };

            trail.push(dir_name);
            ui::render_drill_level(&level, depth, !should_stop);

            if should_stop {
                break;
            }

            current = biggest_dir_path.unwrap();
        }

        ui::render_drill_trail(&trail);

        Ok(NoOutput)
    }
}

fn dir_display_name(path: &PathBuf, root: &PathBuf) -> String {
    if path == root {
        ui::short_path(root)
    } else {
        path.file_name()
            .map(|n| format!("{}/", n.to_string_lossy()))
            .unwrap_or_else(|| ui::short_path(path))
    }
}
