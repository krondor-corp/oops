---
title: How oops works
slug: overview
---

## Philosophy

macOS doesn't ship with `gparted` or anything like it. When your disk fills up, you're stuck clicking through Finder or running `du -sh *` one directory at a time. `oops` fixes that.

The design principles:

- **One command.** `oops` does one thing: show you where your disk space went. No modes to learn, no cleanup engine to configure — just the map, plus `free` and `update`.
- **Fast by default.** Parallel scanning across all CPU cores. Most directories render instantly.
- **Honest sizes.** Reports on-disk block usage, not apparent file sizes. A 1 TiB sparse Docker image that only uses 20 GiB of blocks shows as 20 GiB.
- **Terminal-native.** Colored output with proportional bars — all to stderr, so stdout stays clean for scripting.
- **APFS-aware.** Handles macOS firmlinks correctly when scanning `/` to avoid double-counting.

## What you see

Run `oops` in any directory and you get the map: every entry on its own line, with a bar sized proportionally to how much space it takes. Your eye lands on the biggest thing immediately — no sorting, no reading numbers.

At the top, a volume context bar shows the bigger picture: how much of the disk you scanned, how much is elsewhere, and how much is free. If most of your usage lives outside the directory you scanned, oops tells you where to look next.

```
  / (460.4 GiB total, 27.0 GiB free)
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  scanned 270.6 GiB  other 118.8 GiB  free 71.0 GiB

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   270.6 GiB  ~/
  ━━━━━━━━━━━━━                               91.7 GiB    Library/
  ━━━━━                                       32.8 GiB      Application Support/
  ━━━━━━                                      55.6 GiB    repos/
```

## Key commands

| Command | Purpose |
|---------|---------|
| `oops` (default, no subcommand) | Visual disk usage map — proportional bars, one line per entry |
| `oops free` | One-liner: how much space is left? |
| `oops update` | Self-update to the latest release |

See [Commands]({{ '/docs/commands/' | relative_url }}) for flags and details.

## On-disk sizing

Most disk usage tools report **apparent size** — what the file claims to be. For **sparse files** (like Docker.raw on macOS), the apparent size can be wildly larger than the actual disk blocks allocated.

oops reports allocated blocks — the same metric `du` uses. This gives you the real on-disk footprint.

> A Docker.raw file might report 1 TiB apparent size but only consume 20 GiB of actual disk blocks. oops shows you the 20 GiB.

More detail in [Sparse files]({{ '/docs/sparse-files/' | relative_url }}).

## APFS handling

On macOS, `oops /` redirects to `/System/Volumes/Data` and skips child mount points to avoid double-counting via APFS firmlinks. The volume context bar at the top shows the breakdown: scanned data, system/other, and free space.

## Contributing

oops is open source — the developer docs (architecture, coding patterns, contribution guide) live in the [GitHub repo](https://github.com/krondor-corp/oops/tree/main/docs).
