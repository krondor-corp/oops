---
title: Recipes
slug: recipes
---

Real-world scenarios and the fastest way to diagnose them with oops.

## "My disk is full and I don't know why"

```bash
oops map ~/
```

One command. Shows your full disk breakdown with volume context at the top (scanned / other / free), then drills into the biggest space consumers with proportional bars. You'll immediately see whether it's repos, Library, Docker, or something unexpected.

To see the full disk including system files:

```bash
oops map /
```

## "What can I clean up right now?"

```bash
oops sweep ~/ --exec
```

Finds reclaimable space (build artifacts, caches, node_modules, etc.) and walks you through cleanup interactively — shows each match with the cleanup command and asks y/N before running it.

For a preview without cleaning:

```bash
oops sweep ~/
oops sweep ~/ -v   # Show individual paths
```

## "What's taking up space in this repo?"

```bash
cd ~/repos/my-project
oops map .
```

Usually it's `target/`, `node_modules/`, or `.git/`. For a quick cleanup:

```bash
oops sweep . --exec
```

## "Where are all my node_modules?"

```bash
oops sweep ~/ --rule node_modules -v
```

Lists every `node_modules/` with its path and size. Add `--exec` to interactively remove them:

```bash
oops sweep ~/ --rule node_modules --exec
```

## "Is Docker eating my disk?"

```bash
oops map ~/Library/Containers/com.docker.docker
```

Shows you Docker.raw's actual on-disk size (not the inflated apparent size). If it's huge:

```bash
docker system prune -a
```

## "Xcode is eating 50 GB again"

```bash
oops sweep ~/ --rule xcode -v --exec
```

The `--rule xcode` flag matches both `xcode-derived` and `xcode-simulators` rules. For a full picture:

```bash
oops map ~/Library/Developer
```

Clean up:

```bash
rm -rf ~/Library/Developer/Xcode/DerivedData
xcrun simctl delete unavailable
xcrun simctl runtime delete all
```

## "What's in ~/Library?"

`~/Library` is a black box. Map it:

```bash
oops map ~/Library --depth 3
```

Common offenders:
- `Application Support/` — app data (Claude VMs, Steam games, Slack)
- `Caches/` — safe to delete, apps rebuild them
- `Containers/` — sandboxed app data (Docker lives here)
- `Developer/` — Xcode caches and simulators
- `pnpm/` — pnpm global store

## "Cargo registry is huge"

```bash
oops sweep ~/ --rule cargo -v
```

The registry cache grows with every unique dependency version. Clean old versions:

```bash
cargo cache --autoclean
# or
rm -rf ~/.cargo/registry/cache
```

## "I freed space but disk still shows full"

macOS uses APFS purgeable space. Check with:

```bash
oops free
```

If the volume still shows full after deleting files, Time Machine snapshots may be holding references. macOS will purge these under pressure, or force it:

```bash
tmutil listlocalsnapshots /
tmutil deletelocalsnapshots <date>
```

## "Where do I start on a new machine?"

```bash
oops map ~/              # Where's my space going?
oops sweep ~/ --exec     # Clean up the obvious stuff
oops config              # See and customize sweep rules
```

Three commands. You know your disk layout, you've reclaimed the easy wins, and the tool is configured for next time.

## "I want to track custom waste patterns"

```bash
oops config    # Creates ~/.config/oops/config.toml if needed
```

Edit the config to add rules:

```toml
[[rules]]
name = "jig-worktrees"
dir = "worktrees"
min_size = "100 MB"
clean = "rm -rf {path}"
description = "stale jig worker worktrees"

[[rules]]
name = "dist-builds"
dir = "dist"
when_parent_has = "package.json"
min_size = "50 MB"
clean = "rm -rf {path}"
description = "frontend build output"
```

Now `oops sweep` picks up your custom rules alongside the built-ins.

## "Automated disk monitoring"

Use `--plain` mode for scripts:

```bash
oops free --plain 2>&1 | head -1
```
