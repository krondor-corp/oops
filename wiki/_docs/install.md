---
title: Installation
slug: install
---

## From source

```bash
git clone https://github.com/krondor-corp/oops
cd oops
cargo install --path crates/oops-cli
```

## Requirements

- **Rust** 1.75+ (for building from source)
- **macOS** or **Linux** (uses Unix `stat` for block-level sizing and `df -Pk` for volumes)

## Verify

```bash
oops --version
oops --help
```

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
