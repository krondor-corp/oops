# oops

**Where did all my disk space go?**

Fast disk usage diagnostics for macOS and Unix. Map what's eating your drive, find waste, reclaim space — in seconds.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/krondor-corp/oops/main/install.sh | bash
```

Downloads the latest release binary for your platform into `~/.local/bin` (override with `INSTALL_DIR=...`). Prebuilt binaries ship for `aarch64-darwin` and `x86_64-linux`.

From source:

```bash
cargo install --path crates/oops-cli    # requires Rust 1.75+
```

## Update

```bash
oops update           # upgrade to the latest release
oops update --force   # reinstall even if already current
```

## Usage

```bash
oops                  # Visual disk map of current directory
oops map ~/           # Map of home directory
oops map /            # Full disk (APFS-aware)
oops sweep ~/         # Find reclaimable space (configurable rules)
oops sweep --exec ~/  # Interactive cleanup
oops config           # Inspect/edit rule config
oops free             # One-liner: how full am I?
oops vol              # Mounted volumes with capacity bars
```

## What it looks like

```
  / (460.4 GiB total, 27.0 GiB free)
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  scanned 270.6 GiB  other 118.8 GiB  free 71.0 GiB

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   270.6 GiB  ~/
  ━━━━━━━━━━━━━                               91.7 GiB    Library/
  ━━━━━                                       32.8 GiB      Application Support/
  ━━                                          18.7 GiB      Caches/
  ━━━━━━                                      55.6 GiB    repos/
  ━━━━━                                       45.4 GiB      zim/
```

## Commands

| Command | What it does |
|---------|-------------|
| `oops` / `oops map` | Visual disk map — proportional bars, one entry per line |
| `oops sweep` | Find reclaimable space using configurable rules; `--exec` for interactive cleanup |
| `oops config` | Show or initialize config (`~/.config/oops/config.toml`) and rules |
| `oops free` | One-liner volume free-space summary |
| `oops vol` | Mounted volumes with color-coded capacity bars |
| `oops update` | Self-update to the latest GitHub release |

## Why oops?

- **Fast.** Parallel scanning with rayon. Most directories render instantly.
- **Honest sizes.** Reports on-disk block usage, not apparent file sizes. A 1 TiB sparse Docker image using 20 GiB of blocks shows as 20 GiB.
- **APFS-aware.** `oops map /` handles macOS firmlinks correctly — no double-counting.
- **Configurable cleanup.** Built-in sweep rules plus your own custom patterns via TOML config.

## Global flags

| Flag | Effect |
|------|--------|
| `--plain` | No colors, no decorations — for scripting |
| `-v` / `--verbose` | Debug tracing output to stderr |

## Recipes

**My disk is full and I don't know why:**
```bash
oops map ~/
```

**What can I clean up right now?**
```bash
oops sweep ~/ --exec
```

**Where are all my node_modules?**
```bash
oops sweep ~/ --rule node_modules -v
```

**Is Docker eating my disk?**
```bash
oops map ~/Library/Containers/com.docker.docker
```

**Xcode is eating 50 GB again:**
```bash
oops sweep ~/ --rule xcode --exec
```

## Architecture

```
crates/
├── oops-core/    # Library: scanning, volumes, config, rules, sweep engine
└── oops-cli/     # Binary: commands, UI rendering, terminal output
```

## License

MIT
