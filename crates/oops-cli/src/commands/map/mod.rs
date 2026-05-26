//! Map command -- visual disk usage map

mod render;

use std::path::PathBuf;

use clap::Args;

use oops_core::{list_volumes, scan_tree, ScanOptions, Volume};

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Show where your disk space went
#[derive(Args, Debug, Clone)]
pub struct Map {
    /// Target path to analyze
    pub path: Option<PathBuf>,

    /// Maximum depth levels to display
    #[arg(short, long, default_value = "4")]
    pub depth: usize,

    /// Max entries shown per directory level
    #[arg(short = 'n', long, default_value = "5")]
    pub show: usize,

    /// Max children to recurse into per level
    #[arg(long, default_value = "3")]
    pub expand: usize,

    /// Terminal width override (default: auto-detect)
    #[arg(long)]
    pub width: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Map {
    type Error = MapError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);
        let scan_target = resolve_scan_target(target);
        let canonical = std::fs::canonicalize(&scan_target).unwrap_or_else(|_| scan_target.clone());

        let skip_paths = mount_points_except(&canonical);

        let opts = ScanOptions {
            one_filesystem: true,
            skip_paths,
            ..Default::default()
        };

        let spinner = ui::Spinner::start("scanning...");
        let tree = scan_tree(&scan_target, self.depth, &opts)?;
        spinner.stop();

        let volume = find_volume_for(target);

        let tw = self.width.unwrap_or_else(ui::term_width);

        render::render(
            &tree,
            volume.as_ref(),
            tw,
            self.depth,
            self.show,
            self.expand,
        );

        Ok(NoOutput)
    }
}

fn find_volume_for(path: &std::path::Path) -> Option<Volume> {
    let volumes = list_volumes().ok()?;
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    volumes
        .into_iter()
        .filter(|v| canonical.starts_with(&v.mount_point))
        .max_by_key(|v| v.mount_point.as_os_str().len())
}

/// Return paths to skip during scanning to avoid APFS double counting.
///
/// On macOS with APFS, `/` is a sealed system volume and `/System/Volumes/Data`
/// is the writable data volume. Firmlinks from `/Users`, `/Library`, etc. point
/// into the data volume, so scanning `/` would count data twice. We skip
/// child mount points AND, when scanning `/`, redirect to the data volume.
fn mount_points_except(target: &std::path::Path) -> std::collections::HashSet<std::path::PathBuf> {
    let volumes = match list_volumes() {
        Ok(v) => v,
        Err(_) => return Default::default(),
    };
    volumes
        .iter()
        .filter(|v| v.mount_point != target && v.mount_point.starts_with(target))
        .map(|v| v.mount_point.clone())
        .collect()
}

/// If scanning `/` on macOS, redirect to `/System/Volumes/Data` to avoid
/// double counting via APFS firmlinks.
fn resolve_scan_target(target: &std::path::Path) -> std::path::PathBuf {
    let data_vol = std::path::Path::new("/System/Volumes/Data");
    if target == std::path::Path::new("/") && data_vol.exists() {
        data_vol.to_path_buf()
    } else {
        target.to_path_buf()
    }
}
