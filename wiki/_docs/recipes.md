---
title: Recipes
slug: recipes
---

Real-world scenarios and the fastest way to diagnose them with oops.

## "My disk is full and I don't know why"

Start with the overview, then drill:

```bash
oops
oops drill ~
```

The drill auto-follows the biggest child at each level. In one command you'll trace the path from your home directory to the specific file or directory eating all your space — often Docker data, Xcode DerivedData, or a forgotten game install.

## "What's taking up space in this repo?"

```bash
cd ~/repos/my-project
oops
```

The default overview shows a proportional breakdown. Usually it's `target/`, `node_modules/`, or `.git/`. For deeper visibility:

```bash
oops tree --depth 4
```

## "Is Docker eating my disk?"

```bash
oops drill ~/Library/Containers/com.docker.docker
```

This drills through `Data/vms/0/data/` and shows you `Docker.raw`'s actual on-disk size (not the inflated apparent size). If it's huge:

```bash
# In Docker Desktop: Settings → Resources → Virtual disk limit
# Or prune unused data:
docker system prune -a
```

## "Where are all my node_modules?"

```bash
oops sweep ~
```

Sweep detects `node_modules` directories across your entire home folder. The summary shows total reclaimable space by category. For details:

```bash
oops sweep ~ --verbose
```

This lists every individual waste entry with its path, so you can decide what to nuke:

```bash
# Delete a specific one
rm -rf ~/old-project/node_modules

# Or nuke them all (careful!)
oops sweep ~ --verbose | grep node_modules
```

## "Xcode is eating 50 GB again"

```bash
oops sweep ~
```

Sweep checks `~/Library/Developer/Xcode/DerivedData` and `~/Library/Developer/CoreSimulator` automatically. To see just how bad it is:

```bash
oops drill ~/Library/Developer
```

Clean up:

```bash
rm -rf ~/Library/Developer/Xcode/DerivedData
# Xcode will rebuild what it needs
```

## "What are the biggest files on my machine?"

```bash
oops top ~ --depth 8 -n 30
```

This does a deep recursive scan and shows the 30 largest items. Filter to just files:

```bash
oops top ~ --depth 8 --files-only --min-size 500MB
```

## "Which volume is running out of space?"

```bash
oops vol
```

Color-coded capacity bars: green (< 70%), yellow (< 90%), red (>= 90%). Shows all mounted filesystems with used/total/free.

## "I freed space but disk still shows full"

macOS uses APFS snapshots and purgeable space. Check with:

```bash
oops vol
```

If the volume still shows full after deleting files, Time Machine snapshots may be holding references. macOS will purge these eventually, or you can force it:

```bash
tmutil listlocalsnapshots /
tmutil deletelocalsnapshots <date>
```

## "What's in ~/Library?"

`~/Library` is a black box. Drill into it:

```bash
oops drill ~/Library
```

Common offenders:
- `Application Support/` — app data (Steam games, Slack, etc.)
- `Caches/` — safe to delete, apps rebuild them
- `Containers/` — sandboxed app data (Docker lives here)
- `Developer/` — Xcode caches and simulators
- `pnpm/` — pnpm global store

## "Compare two directories"

Run the overview on each:

```bash
oops /path/to/dir-a
oops /path/to/dir-b
```

Or use top to find the biggest items in each:

```bash
oops top /path/to/dir-a -n 10
oops top /path/to/dir-b -n 10
```

## "Automated disk monitoring"

Use `--plain` mode for scripts:

```bash
# Alert if any volume > 90%
oops vol --plain 2>&1 | awk '{print $5}' | grep -q '9[0-9]%' && echo "DISK ALERT"
```

## "Cargo registry is huge"

```bash
oops drill ~/.cargo
```

The registry cache grows with every unique dependency version you've ever built. Clean old versions:

```bash
cargo cache --autoclean
# or
rm -rf ~/.cargo/registry/cache
```

## "Where do I start on a new machine?"

```bash
oops vol
oops drill ~
oops sweep ~
```

Three commands. You now know your volume health, your biggest space consumer, and all the reclaimable waste. Takes about 30 seconds total.
