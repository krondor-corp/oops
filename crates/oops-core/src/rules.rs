use std::path::{Path, PathBuf};

use crate::config::{OopsConfig, RuleConfig};

#[derive(Debug, Clone)]
pub enum MatchMode {
    DirBasename(String),
    AbsolutePath(PathBuf),
    FileGlob(String),
}

#[derive(Debug, Clone)]
pub struct ResolvedRule {
    pub name: String,
    pub match_mode: MatchMode,
    pub when_parent_has: Option<String>,
    pub min_size: u64,
    pub clean: Option<String>,
    pub description: String,
}

impl ResolvedRule {
    pub fn matches_entry(&self, entry_name: &str, entry_path: &Path, is_dir: bool) -> bool {
        match &self.match_mode {
            MatchMode::DirBasename(name) => is_dir && entry_name == name,
            MatchMode::AbsolutePath(p) => entry_path == p,
            MatchMode::FileGlob(pattern) => {
                if is_dir {
                    return false;
                }
                if let Some(ext) = pattern.strip_prefix("*.") {
                    entry_name
                        .rsplit('.')
                        .next()
                        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
                } else {
                    entry_name == pattern
                }
            }
        }
    }

    pub fn check_parent_condition(&self, parent: &Path) -> bool {
        match &self.when_parent_has {
            Some(required_file) => parent.join(required_file).exists(),
            None => true,
        }
    }

    pub fn is_absolute_path(&self) -> bool {
        matches!(self.match_mode, MatchMode::AbsolutePath(_))
    }

    pub fn expand_clean_command(&self, matched_path: &Path) -> Option<String> {
        self.clean.as_ref().map(|tmpl| {
            tmpl.replace("{path}", &matched_path.display().to_string())
                .replace(
                    "{parent}",
                    &matched_path
                        .parent()
                        .unwrap_or(matched_path)
                        .display()
                        .to_string(),
                )
                .replace(
                    "{name}",
                    &matched_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                )
        })
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn parse_size(s: &str) -> Result<u64, crate::Error> {
    let s = s.trim();
    // bytesize parses "50 MB", "1 GiB", etc.
    s.parse::<bytesize::ByteSize>()
        .map(|b| b.as_u64())
        .map_err(|_| crate::Error::InvalidRule(format!("invalid size: {s}")))
}

fn rule_from_config(rc: &RuleConfig) -> Result<ResolvedRule, crate::Error> {
    let match_mode = if let Some(ref dir) = rc.dir {
        MatchMode::DirBasename(dir.clone())
    } else if let Some(ref path) = rc.path {
        MatchMode::AbsolutePath(expand_tilde(path))
    } else if let Some(ref glob) = rc.file_glob {
        MatchMode::FileGlob(glob.clone())
    } else {
        return Err(crate::Error::InvalidRule(format!(
            "rule '{}': must have exactly one of dir, path, or file_glob",
            rc.name
        )));
    };

    let min_size = match &rc.min_size {
        Some(s) => parse_size(s)?,
        None => 0,
    };

    Ok(ResolvedRule {
        name: rc.name.clone(),
        match_mode,
        when_parent_has: rc.when_parent_has.clone(),
        min_size,
        clean: rc.clean.clone(),
        description: rc.description.clone().unwrap_or_else(|| rc.name.clone()),
    })
}

pub fn builtin_rules() -> Vec<ResolvedRule> {
    let mut rules = vec![
        ResolvedRule {
            name: "node_modules".into(),
            match_mode: MatchMode::DirBasename("node_modules".into()),
            when_parent_has: None,
            min_size: 1024 * 1024,
            clean: Some("rm -rf {path}".into()),
            description: "npm/yarn/pnpm dependency directories".into(),
        },
        ResolvedRule {
            name: "rust-target".into(),
            match_mode: MatchMode::DirBasename("target".into()),
            when_parent_has: Some("Cargo.toml".into()),
            min_size: 50 * 1024 * 1024,
            clean: Some("cargo clean --manifest-path {parent}/Cargo.toml".into()),
            description: "Rust compilation artifacts".into(),
        },
        ResolvedRule {
            name: "cmake-build".into(),
            match_mode: MatchMode::DirBasename("build".into()),
            when_parent_has: Some("CMakeLists.txt".into()),
            min_size: 50 * 1024 * 1024,
            clean: Some("rm -rf {path}".into()),
            description: "CMake build output".into(),
        },
        ResolvedRule {
            name: "python-venv".into(),
            match_mode: MatchMode::DirBasename(".venv".into()),
            when_parent_has: None,
            min_size: 10 * 1024 * 1024,
            clean: Some("rm -rf {path}".into()),
            description: "Python virtual environments".into(),
        },
        ResolvedRule {
            name: "pycache".into(),
            match_mode: MatchMode::DirBasename("__pycache__".into()),
            when_parent_has: None,
            min_size: 1024 * 1024,
            clean: Some("rm -rf {path}".into()),
            description: "Python bytecode cache".into(),
        },
        ResolvedRule {
            name: "log-files".into(),
            match_mode: MatchMode::FileGlob("*.log".into()),
            when_parent_has: None,
            min_size: 10 * 1024 * 1024,
            clean: Some("rm {path}".into()),
            description: "Large log files".into(),
        },
        ResolvedRule {
            name: "dot-cache".into(),
            match_mode: MatchMode::DirBasename(".cache".into()),
            when_parent_has: None,
            min_size: 50 * 1024 * 1024,
            clean: None,
            description: "Cache directories".into(),
        },
    ];

    // macOS-specific absolute path rules
    if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            let macos_rules = vec![
                ResolvedRule {
                    name: "macos-caches".into(),
                    match_mode: MatchMode::AbsolutePath(home.join("Library/Caches")),
                    when_parent_has: None,
                    min_size: 50 * 1024 * 1024,
                    clean: None,
                    description: "macOS application caches".into(),
                },
                ResolvedRule {
                    name: "xcode-derived".into(),
                    match_mode: MatchMode::AbsolutePath(
                        home.join("Library/Developer/Xcode/DerivedData"),
                    ),
                    when_parent_has: None,
                    min_size: 50 * 1024 * 1024,
                    clean: Some("rm -rf {path}".into()),
                    description: "Xcode build cache".into(),
                },
                ResolvedRule {
                    name: "xcode-simulators".into(),
                    match_mode: MatchMode::AbsolutePath(
                        home.join("Library/Developer/CoreSimulator"),
                    ),
                    when_parent_has: None,
                    min_size: 50 * 1024 * 1024,
                    clean: None,
                    description: "iOS Simulator data".into(),
                },
                ResolvedRule {
                    name: "npm-cache".into(),
                    match_mode: MatchMode::AbsolutePath(home.join(".npm/_cacache")),
                    when_parent_has: None,
                    min_size: 50 * 1024 * 1024,
                    clean: Some("npm cache clean --force".into()),
                    description: "npm download cache".into(),
                },
                ResolvedRule {
                    name: "pnpm-store".into(),
                    match_mode: MatchMode::AbsolutePath(home.join("Library/pnpm")),
                    when_parent_has: None,
                    min_size: 50 * 1024 * 1024,
                    clean: Some("pnpm store prune".into()),
                    description: "pnpm content-addressable store".into(),
                },
                ResolvedRule {
                    name: "cargo-registry".into(),
                    match_mode: MatchMode::AbsolutePath(home.join(".cargo/registry")),
                    when_parent_has: None,
                    min_size: 50 * 1024 * 1024,
                    clean: None,
                    description: "Cargo crate registry cache".into(),
                },
                ResolvedRule {
                    name: "docker-data".into(),
                    match_mode: MatchMode::AbsolutePath(
                        home.join("Library/Containers/com.docker.docker"),
                    ),
                    when_parent_has: None,
                    min_size: 100 * 1024 * 1024,
                    clean: None,
                    description: "Docker Desktop data".into(),
                },
            ];
            rules.extend(macos_rules);
        }
    }

    rules
}

pub fn resolve_rules(config: &OopsConfig) -> Result<Vec<ResolvedRule>, crate::Error> {
    let mut rules = builtin_rules();

    for user_rule in &config.rules {
        if let Some(idx) = rules.iter().position(|r| r.name == user_rule.name) {
            if user_rule.enabled == Some(false) {
                rules.remove(idx);
                continue;
            }
            // Override: replace built-in with user version
            let resolved = rule_from_config(user_rule)?;
            rules[idx] = resolved;
        } else {
            if user_rule.enabled == Some(false) {
                continue;
            }
            let resolved = rule_from_config(user_rule)?;
            rules.push(resolved);
        }
    }

    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rules_are_valid() {
        let rules = builtin_rules();
        assert!(!rules.is_empty());
        for rule in &rules {
            assert!(!rule.name.is_empty());
            assert!(!rule.description.is_empty());
        }
    }

    #[test]
    fn resolve_disables_builtin() {
        let config = OopsConfig {
            rules: vec![RuleConfig {
                name: "node_modules".into(),
                enabled: Some(false),
                dir: None,
                path: None,
                file_glob: None,
                when_parent_has: None,
                min_size: None,
                clean: None,
                description: None,
            }],
            ..Default::default()
        };
        let rules = resolve_rules(&config).unwrap();
        assert!(!rules.iter().any(|r| r.name == "node_modules"));
    }

    #[test]
    fn resolve_adds_user_rule() {
        let config = OopsConfig {
            rules: vec![RuleConfig {
                name: "my-custom".into(),
                enabled: None,
                dir: Some("dist".into()),
                path: None,
                file_glob: None,
                when_parent_has: Some("package.json".into()),
                min_size: Some("100 MB".into()),
                clean: Some("rm -rf {path}".into()),
                description: Some("Frontend build output".into()),
            }],
            ..Default::default()
        };
        let rules = resolve_rules(&config).unwrap();
        assert!(rules.iter().any(|r| r.name == "my-custom"));
    }

    #[test]
    fn expand_template() {
        let rule = ResolvedRule {
            name: "test".into(),
            match_mode: MatchMode::DirBasename("target".into()),
            when_parent_has: None,
            min_size: 0,
            clean: Some("cargo clean --manifest-path {parent}/Cargo.toml".into()),
            description: "test".into(),
        };
        let cmd = rule
            .expand_clean_command(Path::new("/home/user/project/target"))
            .unwrap();
        assert_eq!(
            cmd,
            "cargo clean --manifest-path /home/user/project/Cargo.toml"
        );
    }
}
