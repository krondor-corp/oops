---
title: Commands
slug: commands
---

## oops (default)

With no subcommand, `oops` shows the visual disk map of the current (or given) directory — proportional bars, one entry per line. This is the whole point of the tool; there is no separate `map` subcommand.

```bash
oops                       # Where's my space going?
oops /some/path            # Map of a specific path
oops /                     # Full disk breakdown (APFS-aware)
oops --depth 2 ~/          # Shallow overview
oops -n 10 --expand 10 ~/  # Show everything
```

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--depth` | 4 | Maximum depth levels |
| `-n` / `--show` | 5 | Max entries shown per directory level |
| `--expand` | 3 | Max children to recurse into per level |
| `--width` | auto | Terminal width override |

On macOS, `oops /` automatically redirects to the APFS data volume to avoid double-counting via firmlinks. Shows a volume context bar at the top with scanned/other/free breakdown.

Want a shorthand for a common invocation? Alias it in your shell:

```bash
alias oopsd='oops --depth 2'
```

---

## free

One-liner: how much space is left on the volume containing the current (or specified) path.

```bash
oops free
oops free /Volumes/External
```

---

## update

Self-update to the latest release.

```bash
oops update
```
