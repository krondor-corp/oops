use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;
use tracing::{debug, debug_span};

use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;

use crate::disk_size;

#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    /// Stay on the same filesystem (like `du -x`). Default: false.
    pub one_filesystem: bool,
    /// Absolute paths to skip during scanning (e.g. APFS synthetic mounts).
    pub skip_paths: HashSet<PathBuf>,
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
    let _span = debug_span!("scan_directory", path = %path.display(), max_depth = ?opts.max_depth)
        .entered();
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
                    let size =
                        dir_size_recursive(&entry_path, opts, depth + 1, &file_count, &error_count);
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

// ---------------------------------------------------------------------------
// Tree scan — single-pass recursive tree for icicle/map rendering
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
}

/// Build a size-annotated tree up to `max_depth` levels deep.
/// Children beyond `max_depth` are still included in the parent's size
/// but not retained as nodes. Uses rayon for parallelism.
pub fn scan_tree(
    path: &Path,
    max_depth: usize,
    opts: &ScanOptions,
) -> Result<TreeNode, crate::Error> {
    if !path.exists() {
        return Err(crate::Error::PathNotFound(path.display().to_string()));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let root_dev = if opts.one_filesystem {
        fs::symlink_metadata(path).ok().map(|m| m.dev())
    } else {
        None
    };

    Ok(build_tree_node(path, &name, max_depth, 0, opts, root_dev))
}

fn build_tree_node(
    path: &Path,
    name: &str,
    max_depth: usize,
    depth: usize,
    opts: &ScanOptions,
    root_dev: Option<u64>,
) -> TreeNode {
    let meta = if opts.follow_symlinks {
        fs::metadata(path)
    } else {
        fs::symlink_metadata(path)
    };

    let meta = match meta {
        Ok(m) => m,
        Err(_) => {
            return TreeNode {
                name: name.to_string(),
                path: path.to_path_buf(),
                size: 0,
                is_dir: false,
                children: vec![],
            };
        }
    };

    // Skip explicitly excluded paths (e.g. APFS synthetic mounts)
    if !opts.skip_paths.is_empty() && opts.skip_paths.contains(path) {
        return TreeNode {
            name: name.to_string(),
            path: path.to_path_buf(),
            size: 0,
            is_dir: meta.is_dir(),
            children: vec![],
        };
    }

    // Skip entries on a different filesystem
    if let Some(rd) = root_dev {
        if meta.dev() != rd {
            return TreeNode {
                name: name.to_string(),
                path: path.to_path_buf(),
                size: 0,
                is_dir: meta.is_dir(),
                children: vec![],
            };
        }
    }

    if !meta.is_dir() {
        return TreeNode {
            name: name.to_string(),
            path: path.to_path_buf(),
            size: crate::disk_size(&meta),
            is_dir: false,
            children: vec![],
        };
    }

    // Beyond max_depth: aggregate size only, don't retain children
    if depth >= max_depth {
        let file_count = AtomicU64::new(0);
        let error_count = AtomicU64::new(0);
        let size = dir_size_recursive(path, opts, 0, &file_count, &error_count);
        return TreeNode {
            name: name.to_string(),
            path: path.to_path_buf(),
            size,
            is_dir: true,
            children: vec![],
        };
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => {
            return TreeNode {
                name: name.to_string(),
                path: path.to_path_buf(),
                size: 0,
                is_dir: true,
                children: vec![],
            };
        }
    };

    let entries: Vec<PathBuf> = read_dir.filter_map(|e| e.ok().map(|e| e.path())).collect();

    let mut children: Vec<TreeNode> = entries
        .par_iter()
        .map(|p| {
            let child_name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            build_tree_node(p, &child_name, max_depth, depth + 1, opts, root_dev)
        })
        .collect();

    children.sort_by_key(|c| std::cmp::Reverse(c.size));

    let size: u64 = children.iter().map(|c| c.size).sum();

    TreeNode {
        name: name.to_string(),
        path: path.to_path_buf(),
        size,
        is_dir: true,
        children,
    }
}

// ---------------------------------------------------------------------------
// Size helpers
// ---------------------------------------------------------------------------

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
