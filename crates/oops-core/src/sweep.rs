use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;
use tracing::{debug, debug_span};

use crate::disk_size;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WasteCategory {
    NodeModules,
    GitObjects,
    BuildArtifacts,
    CacheFiles,
    LogFiles,
    VirtualEnvs,
    ContainerImages,
    PlatformCache,
}

impl WasteCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NodeModules => "node_modules",
            Self::GitObjects => ".git (large repos)",
            Self::BuildArtifacts => "build artifacts",
            Self::CacheFiles => "caches",
            Self::LogFiles => "log files",
            Self::VirtualEnvs => "virtual environments",
            Self::ContainerImages => "container data",
            Self::PlatformCache => "platform caches",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::NodeModules => "npm/yarn/pnpm dependency directories",
            Self::GitObjects => "Git object stores in large repositories",
            Self::BuildArtifacts => "Rust target/, Go bin/, CMake build/",
            Self::CacheFiles => ".cache directories, pip cache, cargo registry",
            Self::LogFiles => "*.log files and log directories",
            Self::VirtualEnvs => "Python venv/, .venv/, conda envs",
            Self::ContainerImages => "Docker/OCI image layers and volumes",
            Self::PlatformCache => "~/Library/Caches, Xcode DerivedData",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasteEntry {
    pub path: PathBuf,
    pub category: WasteCategory,
    pub size: u64,
}

pub fn sweep_directory(path: &Path, max_depth: usize) -> Result<Vec<WasteEntry>, crate::Error> {
    let _span = debug_span!("sweep", path = %path.display(), max_depth).entered();
    debug!("starting sweep");

    if !path.exists() {
        return Err(crate::Error::PathNotFound(path.display().to_string()));
    }

    let mut results = Vec::new();
    sweep_recursive(path, 0, max_depth, &mut results);

    if let Some(home) = home_hint(path) {
        debug!("checking platform caches");
        check_platform_caches(&home, &mut results);
    }

    results.sort_by(|a, b| b.size.cmp(&a.size));
    debug!(entries = results.len(), "sweep complete");
    Ok(results)
}

fn home_hint(path: &Path) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    if path == home || path.starts_with(&home) {
        Some(home)
    } else {
        None
    }
}

fn check_platform_caches(home: &Path, results: &mut Vec<WasteEntry>) {
    let candidates = [
        (home.join("Library/Caches"), WasteCategory::PlatformCache),
        (home.join("Library/Developer/Xcode/DerivedData"), WasteCategory::BuildArtifacts),
        (home.join("Library/Developer/CoreSimulator"), WasteCategory::PlatformCache),
        (home.join(".cargo/registry"), WasteCategory::CacheFiles),
        (home.join(".npm/_cacache"), WasteCategory::CacheFiles),
        (home.join("Library/Containers/com.docker.docker"), WasteCategory::ContainerImages),
    ];

    for (candidate_path, category) in &candidates {
        if candidate_path.is_dir() {
            if results.iter().any(|r| r.path == *candidate_path) {
                continue;
            }
            let size = quick_dir_size(candidate_path);
            if size > 50 * 1024 * 1024 {
                results.push(WasteEntry {
                    path: candidate_path.clone(),
                    category: category.clone(),
                    size,
                });
            }
        }
    }
}

fn sweep_recursive(path: &Path, depth: usize, max_depth: usize, results: &mut Vec<WasteEntry>) {
    if depth > max_depth {
        return;
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry_path.is_dir();

        if is_dir {
            let category = match name.as_str() {
                "node_modules" => Some(WasteCategory::NodeModules),
                ".git" => {
                    let size = quick_dir_size(&entry_path);
                    if size > 100 * 1024 * 1024 { Some(WasteCategory::GitObjects) } else { None }
                }
                "target" if path.join("Cargo.toml").exists() => Some(WasteCategory::BuildArtifacts),
                "build" if path.join("CMakeLists.txt").exists() => Some(WasteCategory::BuildArtifacts),
                ".cache" => Some(WasteCategory::CacheFiles),
                "venv" | ".venv" | "env" if has_python_marker(path) => Some(WasteCategory::VirtualEnvs),
                "__pycache__" => Some(WasteCategory::CacheFiles),
                _ => None,
            };

            if let Some(cat) = category {
                let size = quick_dir_size(&entry_path);
                if size > 1024 * 1024 {
                    results.push(WasteEntry { path: entry_path.clone(), category: cat, size });
                }
                continue;
            }

            if name.starts_with('.') && name != ".git" {
                continue;
            }

            sweep_recursive(&entry_path, depth + 1, max_depth, results);
        } else if name.ends_with(".log") {
            if let Ok(m) = fs::symlink_metadata(&entry_path) {
                let size = disk_size(&m);
                if size > 10 * 1024 * 1024 {
                    results.push(WasteEntry {
                        path: entry_path,
                        category: WasteCategory::LogFiles,
                        size,
                    });
                }
            }
        }
    }
}

fn has_python_marker(dir: &Path) -> bool {
    dir.join("setup.py").exists()
        || dir.join("pyproject.toml").exists()
        || dir.join("requirements.txt").exists()
}

fn quick_dir_size(path: &Path) -> u64 {
    let counter = AtomicU64::new(0);
    quick_dir_size_inner(path, &counter);
    counter.load(Ordering::Relaxed)
}

fn quick_dir_size_inner(path: &Path, counter: &AtomicU64) {
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    let entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();

    entries.par_iter().for_each(|entry| {
        let meta = match fs::symlink_metadata(entry.path()) {
            Ok(m) => m,
            Err(_) => return,
        };

        if meta.is_dir() {
            quick_dir_size_inner(&entry.path(), counter);
        } else {
            counter.fetch_add(disk_size(&meta), Ordering::Relaxed);
        }
    });
}
