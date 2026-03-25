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

Shows the largest entries with proportional bars, sorted by size. Use `oops vol` to see mounted volumes.

---

## drill

Follow the largest child at each level, drilling down automatically until the space is evenly distributed.

```bash
oops drill                # From current directory
oops drill /Users/al      # From a specific path
oops drill --threshold 10 # Keep going until biggest child < 10%
oops drill -n 8           # Show 8 siblings at each level
oops drill --depth 20     # Go up to 20 levels deep
```

| Flag | Default | Effect |
|------|---------|--------|
| `--threshold` | 25% | Stop when largest child is below this % of parent |
| `-n` / `--show` | 5 | Number of sibling entries to show per level |
| `-d` / `--depth` | 10 | Maximum levels to drill |

Output streams progressively — each level renders as soon as its scan completes. A spinner shows during scanning.

---

## sweep

Scan for common disk space wasters: `node_modules`, build artifacts, caches, Docker data, virtual environments, and macOS platform caches.

```bash
oops sweep
oops sweep /Users/al
oops sweep --verbose      # Show individual entries, not just category totals
oops sweep --depth 8      # Scan deeper
```

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--depth` | 6 | Max depth to scan |
| `-v` / `--verbose` | off | Show individual waste entries |

Categories detected:

| Category | What it matches |
|----------|----------------|
| node_modules | npm/yarn/pnpm dependency trees |
| .git (large) | Git object stores > 100 MB |
| build artifacts | Rust `target/`, CMake `build/` |
| caches | `.cache/`, pip cache, cargo registry |
| log files | `*.log` files > 10 MB |
| virtual envs | Python `venv/`, `.venv/` |
| container data | Docker/OCI layers and volumes |
| platform caches | `~/Library/Caches`, Xcode DerivedData |

---

## tree

Recursive size-weighted directory tree. Shows all entries above a minimum percentage of their parent.

```bash
oops tree
oops tree --depth 5       # Go deeper
oops tree --min-pct 5     # Only show items > 5% of parent
```

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--depth` | 3 | Max depth |
| `--min-pct` | 1.0 | Minimum % of parent to display |

---

## top

Find the N largest files and directories, like a disk-aware `ls`.

```bash
oops top
oops top -n 50            # Top 50
oops top --files-only     # Only files
oops top --dirs-only      # Only directories
oops top --min-size 1GB   # Only items > 1 GB
oops top --depth 8        # Scan deeper
```

| Flag | Default | Effect |
|------|---------|--------|
| `-n` / `--count` | 20 | Number of results |
| `-d` / `--depth` | 5 | Max scan depth |
| `--files-only` | off | Exclude directories |
| `--dirs-only` | off | Exclude files |
| `--min-size` | none | Minimum size filter (e.g. `100MB`, `1GB`) |

---

## volumes / vol

Show all mounted filesystems with capacity bars.

```bash
oops volumes
oops vol
```

Filters out pseudo-filesystems (devfs, map auto, none). Shows mount point, filesystem type, total/used/free space, and a color-coded capacity bar (green < 70%, yellow < 90%, red >= 90%).
