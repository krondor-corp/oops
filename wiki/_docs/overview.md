---
title: How oops works
slug: overview
---

## Philosophy

macOS doesn't ship with `gparted` or anything like it. When your disk fills up, you're stuck clicking through Finder or running `du -sh *` one directory at a time. `oops` fixes that.

The design principles:

- **Fast by default.** Parallel scanning with rayon. Most directories render instantly.
- **Honest sizes.** Reports on-disk block usage, not apparent file sizes. A 1 TiB sparse Docker image that only uses 20 GiB of blocks shows as 20 GiB.
- **Terminal-native.** Colored output with proportional bars and tree drawing — all to stderr. Machine-readable output goes to stdout.
- **APFS-aware.** Handles macOS firmlinks correctly when scanning `/` to avoid double-counting.

## Architecture

oops is a Rust workspace with two crates:

```
crates/
├── oops-core/    # Library: scanning, volumes
└── oops-cli/     # Binary: commands, UI rendering, terminal output
```

**oops-core** handles the heavy lifting:
- `scan_top_entries()` — parallel scan of immediate children with aggregated sizes
- `scan_tree()` — recursive tree with sizes, used by the default map view
- `list_volumes()` — portable volume detection via `df -Pk`, used for volume context and APFS mount handling

**oops-cli** handles presentation:
- Each command implements an `Op` trait with typed errors and output
- Shared rendering lives in a `ui/` module (colors, output, tables, bars)
- Per-command rendering lives in `commands/<cmd>/render.rs`
- A `command_enum!` macro generates the dispatch enum from individual command structs

## Key commands

| Command | Purpose |
|---------|---------|
| `oops` (default, no subcommand) | Visual disk usage map — proportional bars, one line per entry |
| `oops free` | One-liner: how much space is left? |
| `oops update` | Self-update to the latest release |

## On-disk sizing

Most disk usage tools report **apparent size** — what `metadata.len()` returns. For **sparse files** (like Docker.raw on macOS), the apparent size can be wildly larger than the actual disk blocks allocated.

oops uses `stat.blocks * 512` — the same metric `du` reports by default. This gives you the real on-disk footprint.

> A Docker.raw file might report 1 TiB apparent size but only consume 20 GiB of actual disk blocks. oops shows you the 20 GiB.

## APFS handling

On macOS, `oops /` redirects to `/System/Volumes/Data` and skips child mount points to avoid double-counting via APFS firmlinks. The volume context bar at the top shows the breakdown: scanned data, system/other, and free space.
