# oops
<!-- daemon spawn verified -->

**Where did all my disk space go?**

Fast disk usage diagnostics for macOS and Unix. See what's eating your drive in seconds.

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
oops                       # Visual disk map of current directory
oops ~/                    # Map of home directory
oops /                     # Full disk (APFS-aware)
oops --depth 2 ~/          # Shallow overview
oops -n 10 --expand 10 ~/  # Show everything
oops free                  # One-liner: how full am I?
```

Want a shorthand for a common invocation? Alias it in your shell:

```bash
alias oopsd='oops --depth 2'
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
| `oops` | Visual disk map — proportional bars, one entry per line |
| `oops free` | One-liner free-space summary for the current volume |
| `oops update` | Self-update to the latest GitHub release |

## Why oops?

- **Fast.** Parallel scanning with rayon. Most directories render instantly.
- **Honest sizes.** Reports on-disk block usage, not apparent file sizes. A 1 TiB sparse Docker image using 20 GiB of blocks shows as 20 GiB.
- **APFS-aware.** `oops /` handles macOS firmlinks correctly — no double-counting.

## Global flags

| Flag | Effect |
|------|--------|
| `-d` / `--depth` | Maximum depth levels to display (default 4) |
| `-n` / `--show` | Max entries shown per directory level (default 5) |
| `--expand` | Max children to recurse into per level (default 3) |
| `--width` | Terminal width override (default: auto-detect) |
| `--plain` | No colors, no decorations — for scripting |
| `-v` / `--verbose` | Debug tracing output to stderr |

## Recipes

**My disk is full and I don't know why:**
```bash
oops ~/
```

**Is Docker eating my disk?**
```bash
oops ~/Library/Containers/com.docker.docker
```

**`~/Library` is a black box:**
```bash
oops ~/Library --depth 3
```

## Architecture

```
crates/
├── oops-core/    # Library: scanning, volumes
└── oops-cli/     # Binary: commands, UI rendering, terminal output
```

## License

MIT
