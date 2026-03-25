---
title: Sparse files
slug: sparse-files
---

## The problem

Run `ls -lh` on Docker's virtual disk:

```
-rw-r--r--  1 al  staff  1.0T  Docker.raw
```

One terabyte? On a 460 GB drive? Something doesn't add up.

`Docker.raw` is a **sparse file**. It has a logical size of 1 TiB (the maximum size Docker Desktop allocated for its virtual disk), but most of that space is empty — never written to. The filesystem only allocates blocks for the parts that actually contain data.

## Apparent vs. on-disk size

```
apparent size:  1,099,511,627,776 bytes  (1.0 TiB)
on-disk size:          22,388,580,352 bytes  (20.9 GiB)
```

Most disk usage tools — including naive Rust code using `metadata.len()` — report the **apparent size**. This is misleading because it counts space that was never allocated.

The actual disk usage comes from the filesystem's block allocation count:

```bash
stat -f "blocks=%b" Docker.raw
# blocks=43727696

# Actual size: 43727696 * 512 = 22,388,580,352 bytes = 20.9 GiB
```

## How oops handles this

oops uses `stat.blocks * 512` for every file, matching what `du` reports by default. This means:

- **Sparse files** show their real footprint, not their inflated logical size
- **Compressed files** (on APFS) may show smaller than their logical size
- **Regular files** show the same size either way (block allocation ≈ logical size)

As a fallback, if `blocks` is zero (some virtual filesystems), oops falls back to the logical size.

## Common sparse files on macOS

| File | Location | Apparent | Typical actual |
|------|----------|----------|----------------|
| Docker.raw | `~/Library/Containers/com.docker.docker/Data/vms/0/data/` | 1+ TiB | 5–60 GiB |
| Time Machine snapshots | Various | Varies | Much smaller |
| APFS sparse disk images | `.sparseimage` files | Configured max | Actual content |

## Why this matters

Without block-level sizing, a tool can report your home directory as 1.3 TiB on a 460 GiB drive. With it, you get the real number — typically 200–300 GiB for a developer workstation. The inflated number makes every other size look proportionally tiny, hiding the directories that are actually consuming your disk.
