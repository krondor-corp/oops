# oops

**Where did all my disk space go?**

Fast disk usage diagnostics for macOS and Unix. Drill into what's eating your drive, find waste, reclaim space -- in seconds.

## Install

```bash
cargo install --path crates/oops-cli
```

Requires Rust 1.75+.

## Usage

```bash
oops                  # Size breakdown of current directory
oops drill ~          # Auto-follow the biggest child at each level
oops sweep ~          # Find reclaimable waste (node_modules, caches, build artifacts)
oops tree             # Recursive size-weighted directory tree
oops top -n 30        # 30 largest files and directories
oops vol              # Mounted volumes with capacity bars
```

## What it looks like

```
~/repos/oops (1.2 GiB)

  target/       879.4 MiB  71.4%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
  .git/         194.8 MiB  15.8%  ▓▓▓▓
  jig/          142.1 MiB  11.5%  ▓▓▓
  wiki/           8.2 MiB   0.7%
  crates/         6.4 MiB   0.5%
```

## Commands

| Command | What it does |
|---------|-------------|
| `oops` | Proportional size breakdown of a directory |
| `oops drill` | Auto-follow the largest child at each level until space is evenly distributed |
| `oops sweep` | Detect node_modules, build artifacts, caches, Docker data, platform cruft |
| `oops tree` | Recursive size-weighted directory tree |
| `oops top` | Top-N largest files and directories |
| `oops vol` | Mounted volumes with color-coded capacity bars |

## Why oops?

- **Fast.** Parallel scanning with rayon. Most directories render instantly.
- **Honest sizes.** Reports on-disk block usage, not apparent file sizes. A 1 TiB sparse Docker image that only uses 20 GiB of blocks shows as 20 GiB.
- **Terminal-native.** Colored output with progress bars, tree drawing, and proportional bars. Machine-readable output goes to stdout.
- **No config needed.** Just run `oops`.

## Global flags

| Flag | Effect |
|------|--------|
| `--plain` | No colors, no decorations -- for scripting |
| `-v` / `--verbose` | Debug tracing output to stderr |

## Recipes

**My disk is full and I don't know why:**
```bash
oops drill ~
```

**Where are all my node_modules?**
```bash
oops sweep ~
```

**What are the biggest files on my machine?**
```bash
oops top ~ --depth 8 --files-only --min-size 500MB
```

**Is Docker eating my disk?**
```bash
oops drill ~/Library/Containers/com.docker.docker
```

**Xcode is eating 50 GB again:**
```bash
oops drill ~/Library/Developer
```

## Architecture

```
crates/
├── oops-core/    # Library: scanning, volumes, waste detection
└── oops-cli/     # Binary: commands, UI rendering, terminal output
```

## License

MIT
# oops
