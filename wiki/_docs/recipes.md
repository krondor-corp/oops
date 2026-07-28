---
title: Recipes
slug: recipes
---

Real-world scenarios and the fastest way to diagnose them with oops.

## "My disk is full and I don't know why"

```bash
oops ~/
```

One command. Shows your full disk breakdown with volume context at the top (scanned / other / free), then drills into the biggest space consumers with proportional bars. You'll immediately see whether it's repos, Library, Docker, or something unexpected.

To see the full disk including system files:

```bash
oops /
```

## "What's taking up space in this repo?"

```bash
cd ~/repos/my-project
oops .
```

Usually it's `target/`, `node_modules/`, or `.git/`. Clean up with the tool that knows how to regenerate it:

```bash
cargo clean            # target/
rm -rf node_modules     # re-fetched by npm/yarn/pnpm install
```

## "Where are all my node_modules?"

```bash
oops ~/ --depth 8 -n 20
```

Widen the map until the `node_modules/` directories show up as their own line, or just target a known repos root directly:

```bash
oops ~/repos --depth 3
```

## "Is Docker eating my disk?"

```bash
oops ~/Library/Containers/com.docker.docker
```

Shows you Docker.raw's actual on-disk size (not the inflated apparent size). If it's huge:

```bash
docker system prune -a
```

## "Xcode is eating 50 GB again"

```bash
oops ~/Library/Developer
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
oops ~/Library --depth 3
```

Common offenders:
- `Application Support/` — app data (Claude VMs, Steam games, Slack)
- `Caches/` — safe to delete, apps rebuild them
- `Containers/` — sandboxed app data (Docker lives here)
- `Developer/` — Xcode caches and simulators
- `pnpm/` — pnpm global store

## "Cargo registry is huge"

```bash
oops ~/.cargo/registry
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
oops ~/
```

One command. You immediately see your disk layout and where to focus cleanup — repos, caches, or Docker.

## "Automated disk monitoring"

Use `--plain` mode for scripts:

```bash
oops free --plain 2>&1 | head -1
```
