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
# Drill into the biggest space hog
oops drill

# Find reclaimable waste (node_modules, caches, etc.)
oops sweep

# See the full size tree
oops tree
```

## Flags

| Flag | Effect |
|------|--------|
| `--plain` | No colors, no decorations — for scripting |
| `-v` / `--verbose` | Debug tracing output to stderr |

These are global — they work with any subcommand.
