---
title: How scanning works
slug: how-it-works
---

## Parallel scanning

oops uses [rayon](https://docs.rs/rayon) for parallel directory traversal. When scanning a directory's children, each child is sized in parallel across available CPU cores.

For a directory with 100 children, all 100 size calculations happen concurrently. Within each child directory, the recursive traversal also parallelizes at each level. This makes scanning large directories (like your home directory) significantly faster than serial tools.

## The scan pipeline

Every command follows the same pattern:

1. **Read the directory** — `fs::read_dir()` collects immediate children
2. **Size each child in parallel** — rayon maps over children, recursively summing block sizes
3. **Sort by size** — largest first
4. **Render** — pass the sorted entries to the `ui` module

The `ui` module owns all rendering. Commands never write to the terminal directly — they compute data and hand it off. This keeps commands thin and testable.

## Block-level sizing

Every file size is computed via `stat.blocks * 512`, not `metadata.len()`. This gives you the actual on-disk footprint:

```
metadata.len()   → logical/apparent size (what the file "says" it is)
stat.blocks * 512 → physical/allocated size (what the disk actually stores)
```

For regular files these are nearly identical. For sparse files, compressed files, or files with holes, they can differ dramatically.

## Volume detection

oops parses `df -Pk` output (POSIX mode, 1K blocks) to know which volumes exist. This gives a predictable 6-column layout across macOS and Linux, avoiding the extra inode columns that macOS `df` normally includes. Pseudo-filesystems (devfs, map auto_home, none) and zero-size mounts are filtered out at parse time.

Volume information powers two things: the context bar at the top of the map (total / free / scanned breakdown for the volume you're scanning) and `oops free`. The map also uses the mount-point list to skip child mounts during a scan, which is what makes scanning `/` on APFS give honest numbers.
