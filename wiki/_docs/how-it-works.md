---
title: How scanning works
slug: how-it-works
---

This page explains why oops is fast and why its numbers are trustworthy. You don't need any of it to use the tool — it's here for the curious.

## Parallel scanning

oops sizes directories in parallel across all your CPU cores. For a directory with 100 children, all 100 size calculations happen concurrently, and the traversal inside each child parallelizes too.

This is why scanning your entire home directory takes seconds, where serial tools like `du` can take minutes on the same hardware.

## Block-level sizing

Every file is measured by the disk blocks it actually occupies, not the size it claims to be:

```
apparent size   → what the file "says" it is (what ls -l shows)
allocated size  → what the disk actually stores (what oops shows)
```

For regular files these are nearly identical. For sparse files, compressed files, or files with holes, they can differ dramatically — see [Sparse files]({{ '/docs/sparse-files/' | relative_url }}) for the classic Docker.raw example.

## Volume detection

oops asks the system (`df`) which volumes are mounted and how full they are. That powers two things you see:

- the context bar at the top of the map — total / free / scanned breakdown for the volume you're on
- `oops free` — the one-liner free-space summary

The mount-point list is also what lets the map skip other volumes mid-scan, so scanning `/` on a Mac gives honest numbers instead of double-counting APFS firmlinks.
