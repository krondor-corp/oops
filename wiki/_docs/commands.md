---
title: Commands
slug: commands
---

## oops (default)

Run with no subcommand to get the overview: a proportional size breakdown of the current directory.

```bash
oops              # Current directory
oops /some/path   # Specific path
```

Shows the largest entries with proportional bars, sorted by size.

---

## map

Visual disk usage map — shows where your space went with proportional bars, one entry per line.

```bash
oops map ~/                     # Where's my space going?
oops map /                      # Full disk breakdown (APFS-aware)
oops map --depth 2 ~/           # Shallow overview
oops map -n 10 --expand 10 ~/   # Show everything
oops map ~/Library/Caches       # Drill into a specific suspect
```

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--depth` | 4 | Maximum depth levels |
| `-n` / `--show` | 5 | Max entries shown per directory level |
| `--expand` | 3 | Max children to recurse into per level |
| `--width` | auto | Terminal width override |

On macOS, `oops map /` automatically redirects to the APFS data volume to avoid double-counting via firmlinks. Shows a volume context bar at the top with scanned/other/free breakdown.

---

## sweep

Find reclaimable disk space using configurable rules. Ships with 14 built-in rules; extend via `~/.config/oops/config.toml`.

```bash
oops sweep ~/                   # Scan home directory
oops sweep ~/ -v                # Show individual entries
oops sweep ~/ --exec            # Interactive cleanup (y/N/a/q per match)
oops sweep ~/ --rule rust-target  # Filter to specific rule(s)
```

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--depth` | 6 | Max depth to scan |
| `-v` / `--verbose` | off | Show individual entries |
| `-e` / `--exec` | off | Interactively confirm and execute cleanup commands |
| `--rule` | all | Filter to specific rule name(s) |

Built-in rules:

| Rule | Matches | Has cleanup command |
|------|---------|:---:|
| node_modules | `node_modules/` directories | ✓ |
| rust-target | `target/` when `Cargo.toml` in parent | ✓ |
| cmake-build | `build/` when `CMakeLists.txt` in parent | ✓ |
| python-venv | `.venv/` directories | ✓ |
| pycache | `__pycache__/` directories | ✓ |
| log-files | `*.log` files > 10 MB | ✓ |
| dot-cache | `.cache/` directories | — |
| macos-caches | `~/Library/Caches` | — |
| xcode-derived | `~/Library/Developer/Xcode/DerivedData` | ✓ |
| xcode-simulators | `~/Library/Developer/CoreSimulator` | — |
| npm-cache | `~/.npm/_cacache` | ✓ |
| pnpm-store | `~/Library/pnpm` | ✓ |
| cargo-registry | `~/.cargo/registry` | — |
| docker-data | `~/Library/Containers/com.docker.docker` | — |

Rules marked with ✓ can be auto-cleaned with `--exec`. Rules without a cleanup command are reported but must be cleaned manually.

---

## config

Inspect and manage oops configuration. Creates the config file automatically if it doesn't exist.

```bash
oops config                     # Show config path + loaded rules
oops config init                # Explicitly create default config
```

Config lives at `$XDG_CONFIG_HOME/oops/config.toml` (default: `~/.config/oops/config.toml`).

Add custom rules or disable built-ins:

```toml
# Add a custom rule
[[rules]]
name = "jig-worktrees"
dir = "worktrees"
min_size = "100 MB"
clean = "rm -rf {path}"
description = "stale jig worker worktrees"

# Disable a built-in
[[rules]]
name = "node_modules"
enabled = false
```

Rule match modes (use exactly one per rule):
- `dir = "name"` — match directories by basename
- `path = "~/some/path"` — match a specific absolute path (tilde expanded)
- `file_glob = "*.ext"` — match files by extension

Optional fields:
- `when_parent_has = "filename"` — parent directory must contain this file
- `min_size = "50 MB"` — skip entries below this threshold
- `clean = "command {path}"` — cleanup command template (`{path}`, `{parent}`, `{name}`)
- `description = "..."` — human-readable description

---

## drill

Follow the largest child at each level, drilling down automatically until the space is evenly distributed.

```bash
oops drill ~/
oops drill --threshold 10      # Keep going until biggest child < 10%
oops drill -n 8                # Show 8 siblings at each level
```

| Flag | Default | Effect |
|------|---------|--------|
| `--threshold` | 25% | Stop when largest child is below this % of parent |
| `-n` / `--show` | 5 | Number of sibling entries to show per level |
| `-d` / `--depth` | 10 | Maximum levels to drill |

---

## top

Find the N largest files and directories.

```bash
oops top ~/
oops top -n 50
oops top --files-only
oops top --min-size 1GB
```

| Flag | Default | Effect |
|------|---------|--------|
| `-n` / `--count` | 20 | Number of results |
| `-d` / `--depth` | 5 | Max scan depth |
| `--files-only` | off | Exclude directories |
| `--dirs-only` | off | Exclude files |
| `--min-size` | none | Minimum size filter |

---

## free

One-liner: how much space is left on the volume containing the current (or specified) path.

```bash
oops free
oops free /Volumes/External
```

---

## volumes / vol

Show all mounted filesystems with capacity bars.

```bash
oops vol
```

Color-coded: green (< 70%), yellow (< 90%), red (>= 90%).

---

## tree

Recursive size-weighted directory tree.

```bash
oops tree --depth 5
oops tree --min-pct 5
```

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--depth` | 3 | Max depth |
| `--min-pct` | 1.0 | Minimum % of parent to display |

---

## update

Self-update to the latest release.

```bash
oops update
```
