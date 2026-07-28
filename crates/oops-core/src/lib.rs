mod scan;
mod volume;

pub use scan::{scan_directory, scan_top_entries, scan_tree, DirEntry, ScanOptions, TreeNode};
pub use volume::{list_volumes, Volume};

use std::os::unix::fs::MetadataExt;
use thiserror::Error;

fn disk_size(meta: &std::fs::Metadata) -> u64 {
    let blocks = meta.blocks();
    if blocks > 0 {
        blocks * 512
    } else {
        meta.len()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("path not found: {0}")]
    PathNotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
}
