use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;
use tracing::{debug, debug_span};

use crate::disk_size;

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ScanOptions {
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
}


#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub file_count: u64,
    pub error_count: u64,
}

/// Scan immediate children of a directory with aggregated sizes.
pub fn scan_top_entries(path: &Path, opts: &ScanOptions) -> Result<Vec<DirEntry>, crate::Error> {
    let _span = debug_span!("scan_top_entries", path = %path.display()).entered();
    debug!("scanning top-level entries");

    if !path.exists() {
        return Err(crate::Error::PathNotFound(path.display().to_string()));
    }

    let read_dir = fs::read_dir(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            crate::Error::PermissionDenied(path.display().to_string())
        } else {
            crate::Error::Io(e)
        }
    })?;

    let entries: Vec<PathBuf> = read_dir.filter_map(|e| e.ok().map(|e| e.path())).collect();

    let results: Vec<DirEntry> = entries
        .par_iter()
        .map(|entry_path| {
            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let meta = if opts.follow_symlinks {
                fs::metadata(entry_path)
            } else {
                fs::symlink_metadata(entry_path)
            };

            match meta {
                Ok(m) => {
                    if m.is_dir() {
                        let file_count = AtomicU64::new(0);
                        let error_count = AtomicU64::new(0);
                        let size =
                            dir_size_recursive(entry_path, opts, 1, &file_count, &error_count);
                        DirEntry {
                            path: entry_path.clone(),
                            name,
                            size,
                            is_dir: true,
                            file_count: file_count.load(Ordering::Relaxed),
                            error_count: error_count.load(Ordering::Relaxed),
                        }
                    } else {
                        DirEntry {
                            path: entry_path.clone(),
                            name,
                            size: disk_size(&m),
                            is_dir: false,
                            file_count: 1,
                            error_count: 0,
                        }
                    }
                }
                Err(_) => DirEntry {
                    path: entry_path.clone(),
                    name,
                    size: 0,
                    is_dir: false,
                    file_count: 0,
                    error_count: 1,
                },
            }
        })
        .collect();

    Ok(results)
}

/// Scan a directory recursively, returning a flat list of all entries up to max_depth.
pub fn scan_directory(path: &Path, opts: &ScanOptions) -> Result<Vec<DirEntry>, crate::Error> {
    let _span = debug_span!("scan_directory", path = %path.display(), max_depth = ?opts.max_depth).entered();
    debug!("starting recursive scan");

    if !path.exists() {
        return Err(crate::Error::PathNotFound(path.display().to_string()));
    }

    let mut results = Vec::new();
    scan_recursive(path, opts, 0, &mut results);
    debug!(entries = results.len(), "scan complete");
    Ok(results)
}

fn scan_recursive(path: &Path, opts: &ScanOptions, depth: usize, results: &mut Vec<DirEntry>) {
    if let Some(max) = opts.max_depth {
        if depth > max {
            return;
        }
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let meta = if opts.follow_symlinks {
            fs::metadata(&entry_path)
        } else {
            fs::symlink_metadata(&entry_path)
        };

        match meta {
            Ok(m) => {
                if m.is_dir() {
                    let file_count = AtomicU64::new(0);
                    let error_count = AtomicU64::new(0);
                    let size = dir_size_recursive(
                        &entry_path,
                        opts,
                        depth + 1,
                        &file_count,
                        &error_count,
                    );
                    results.push(DirEntry {
                        path: entry_path.clone(),
                        name,
                        size,
                        is_dir: true,
                        file_count: file_count.load(Ordering::Relaxed),
                        error_count: error_count.load(Ordering::Relaxed),
                    });
                    if opts.max_depth.is_none_or(|max| depth < max) {
                        scan_recursive(&entry_path, opts, depth + 1, results);
                    }
                } else {
                    results.push(DirEntry {
                        path: entry_path,
                        name,
                        size: disk_size(&m),
                        is_dir: false,
                        file_count: 1,
                        error_count: 0,
                    });
                }
            }
            Err(_) => {
                results.push(DirEntry {
                    path: entry_path,
                    name,
                    size: 0,
                    is_dir: false,
                    file_count: 0,
                    error_count: 1,
                });
            }
        }
    }
}

fn dir_size_recursive(
    path: &Path,
    opts: &ScanOptions,
    depth: usize,
    file_count: &AtomicU64,
    error_count: &AtomicU64,
) -> u64 {
    if let Some(max) = opts.max_depth {
        if depth > max {
            return 0;
        }
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => {
            error_count.fetch_add(1, Ordering::Relaxed);
            return 0;
        }
    };

    let entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();

    entries
        .par_iter()
        .map(|entry| {
            let meta = if opts.follow_symlinks {
                fs::metadata(entry.path())
            } else {
                fs::symlink_metadata(entry.path())
            };

            match meta {
                Ok(m) => {
                    if m.is_dir() {
                        dir_size_recursive(&entry.path(), opts, depth + 1, file_count, error_count)
                    } else {
                        file_count.fetch_add(1, Ordering::Relaxed);
                        disk_size(&m)
                    }
                }
                Err(_) => {
                    error_count.fetch_add(1, Ordering::Relaxed);
                    0
                }
            }
        })
        .sum()
}
