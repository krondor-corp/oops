---
title: How oops works
slug: overview
---

## Philosophy

macOS doesn't ship with `gparted` or anything like it. When your disk fills up, you're stuck clicking through Finder or running `du -sh *` one directory at a time. `oops` fixes that.

The design principles:

- **Fast by default.** Parallel scanning with rayon. Most directories render instantly.
- **Honest sizes.** Reports on-disk block usage, not apparent file sizes. A 1 TiB sparse Docker image that only uses 20 GiB of blocks shows as 20 GiB.
- **Terminal-native.** Colored output with progress bars, tree drawing, and proportional bars — all to stderr. Machine-readable output goes to stdout.
- **No config needed.** Run `oops` and get a directory breakdown immediately. Use `oops vol` when you need volume info.

## Architecture

oops is a Rust workspace with two crates:

```
crates/
├── oops-core/    # Library: scanning, volumes, waste detection
└── oops-cli/     # Binary: commands, UI rendering, terminal output
```

**oops-core** handles the heavy lifting:
- `scan_top_entries()` — parallel scan of immediate children with aggregated sizes
- `scan_directory()` — recursive flat scan up to a max depth
- `list_volumes()` — portable volume detection via `df -Pk`
- `sweep_directory()` — waste pattern matching (node_modules, caches, build artifacts)

**oops-cli** handles presentation:
- Each command implements an `Op` trait with typed errors and output
- All rendering lives in a single `ui` module — commands never format output directly
- A `command_enum!` macro generates the dispatch enum from individual command structs

## On-disk sizing

Most disk usage tools report **apparent size** — what `metadata.len()` returns. This is the logical file size. For normal files, this is fine. But for **sparse files** (like Docker.raw on macOS), the apparent size can be wildly larger than the actual disk blocks allocated.

oops uses `stat.blocks * 512` — the same metric `du` reports by default. This gives you the real on-disk footprint.

> A Docker.raw file might report 1 TiB apparent size but only consume 20 GiB of actual disk blocks. oops shows you the 20 GiB.
