use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;

use crate::disk_size;
use crate::rules::ResolvedRule;

#[derive(Debug, Clone)]
pub struct SweepMatch {
    pub path: PathBuf,
    pub rule_name: String,
    pub description: String,
    pub size: u64,
    pub clean_command: Option<String>,
}

pub fn sweep_directory(
    path: &Path,
    rules: &[ResolvedRule],
    max_depth: usize,
) -> Result<Vec<SweepMatch>, crate::Error> {
    if !path.exists() {
        return Err(crate::Error::PathNotFound(path.display().to_string()));
    }

    let mut results = Vec::new();

    // Walk-based rules (DirBasename, FileGlob)
    let walk_rules: Vec<&ResolvedRule> = rules.iter().filter(|r| !r.is_absolute_path()).collect();
    sweep_recursive(path, &walk_rules, 0, max_depth, &mut results);

    // Absolute path rules — check directly
    for rule in rules.iter().filter(|r| r.is_absolute_path()) {
        if let crate::rules::MatchMode::AbsolutePath(ref target) = rule.match_mode {
            if !target.starts_with(path) && !path.starts_with(target) {
                // Only check if the absolute path is under the scan root,
                // or the scan root is under the absolute path
                if !target.exists() {
                    continue;
                }
            }
            if target.exists() {
                let size = quick_dir_size(target);
                if size >= rule.min_size {
                    results.push(SweepMatch {
                        path: target.clone(),
                        rule_name: rule.name.clone(),
                        description: rule.description.clone(),
                        size,
                        clean_command: rule.expand_clean_command(target),
                    });
                }
            }
        }
    }

    results.sort_by_key(|m| std::cmp::Reverse(m.size));
    Ok(results)
}

fn sweep_recursive(
    path: &Path,
    rules: &[&ResolvedRule],
    depth: usize,
    max_depth: usize,
    results: &mut Vec<SweepMatch>,
) {
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

        // Test against each rule
        let mut matched = false;
        for rule in rules {
            if rule.matches_entry(&name, &entry_path, is_dir) && rule.check_parent_condition(path) {
                let size = if is_dir {
                    quick_dir_size(&entry_path)
                } else {
                    fs::symlink_metadata(&entry_path)
                        .map(|m| disk_size(&m))
                        .unwrap_or(0)
                };

                if size >= rule.min_size {
                    results.push(SweepMatch {
                        path: entry_path.clone(),
                        rule_name: rule.name.clone(),
                        description: rule.description.clone(),
                        size,
                        clean_command: rule.expand_clean_command(&entry_path),
                    });
                }
                matched = true;
                break;
            }
        }

        if matched {
            continue;
        }

        // Recurse into unmatched directories (skip hidden dirs)
        if is_dir && !name.starts_with('.') {
            sweep_recursive(&entry_path, rules, depth + 1, max_depth, results);
        }
    }
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
