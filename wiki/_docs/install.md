---
title: Installation
slug: install
---

## Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/krondor-corp/oops/main/install.sh | bash
```

The script detects your platform, downloads the matching release tarball from GitHub, and drops the `oops` binary into `~/.local/bin`. Override the destination with `INSTALL_DIR=...` if needed.

Prebuilt binaries ship for `aarch64-darwin` (Apple Silicon) and `x86_64-linux`. Other platforms must build from source.

## Requirements

- **macOS** (Apple Silicon) or **Linux** (x86_64) for prebuilt binaries
- `curl` or `wget` (the installer uses whichever is available)
- Rust 1.75+ only if you're building from source

## From source

Only needed for unsupported architectures or if you want to hack on oops:

```bash
git clone https://github.com/krondor-corp/oops
cd oops
make install
```

## Verify

```bash
oops --version
oops --help
```

## Updating

```bash
oops update           # upgrade to the latest release
oops update --force   # reinstall even if already current
```

`oops update` detects how it was installed and, for install-script setups, re-runs the installer in place. Cargo/source builds are prompted before switching to a release binary.

## Quick start

Just run `oops` in any directory:

```bash
oops
```

You'll get a size breakdown of the current directory. From there:

```bash
# See where your space went
oops ~/

# Shallow overview
oops --depth 2 ~/

# One-liner: how full am I?
oops free
```

## Flags

| Flag | Effect |
|------|--------|
| `-d` / `--depth` | Maximum depth levels to display (default 4) |
| `-n` / `--show` | Max entries shown per directory level (default 5) |
| `--expand` | Max children to recurse into per level (default 3) |
| `--width` | Terminal width override (default: auto-detect) |
| `--plain` | No colors, no decorations — for scripting |
| `-v` / `--verbose` | Debug tracing output to stderr |

`--plain` and `-v` are global — they work with any subcommand. The depth/show/expand/width flags apply to the default map view; alias a shorthand in your shell if you want a memorable name for a common combination (e.g. `alias oopsd='oops --depth 2'`).
