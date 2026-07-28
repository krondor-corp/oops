# Project Guide

Fast disk usage diagnostics for macOS and Unix.

## Quick Reference

```bash
cargo build              # Build all crates
cargo test               # Run all tests
cargo clippy             # Lint
cargo fmt --check        # Check formatting
cargo fmt                # Fix formatting
make check               # All of the above
make install             # Install oops binary (cargo install --path)
make wiki                # Start wiki dev server
```

## Releases

Automated off conventional commits to `main`. See `docs/CONTRIBUTING.md` for the full flow. Key pieces:

- `install.sh` -- `curl | bash` installer that pulls the right artifact from GitHub Releases.
- `oops update` -- self-update subcommand (re-runs `install.sh`).
- `.github/workflows/release-pr.yml` -> `release-tag.yml` -> `release.yml` -- the three-stage pipeline driving PR -> tag -> build.
- Requires `RELEASE_PAT` repo secret (fine-grained PAT with `contents:write` + `workflows`) so the tag push can trigger the build.

## Commands

`free`, `update`. Running `oops` with no subcommand shows the visual disk map of the current (or given) directory — this is the default and only way to invoke the map view; there is no `map` subcommand. Map's display is tuned with top-level flags (`-d`/`--depth`, `-n`/`--show`, `--expand`, `--width`); alias a shorthand in your shell if you want one (e.g. `alias oopsd='oops --depth 2'`).

## Project Structure

```
crates/
├── oops-core/     # Library: scanning, volumes
│   ├── scan.rs    # Parallel directory scanning + TreeNode (rayon)
│   ├── volume.rs  # Volume detection via df -Pk
│   └── lib.rs     # Public API + disk_size() helper
└── oops-cli/      # Binary: commands, UI rendering
    ├── commands/   # One file/dir per subcommand (Op trait)
    │   └── map/    # default-view scan+render logic, invoked directly by main.rs
    ├── ui/         # Shared rendering (colors, output, bars, tables)
    ├── op.rs       # Op trait + Ctx + command_enum! macro
    └── main.rs     # Entry point

wiki/              # Jekyll docs site (GitHub Pages)
docs/              # Developer documentation
```

## Documentation

- `docs/index.md` -- Documentation hub and agent instructions
- `docs/PATTERNS.md` -- Coding conventions
- `docs/SUCCESS_CRITERIA.md` -- CI checks
- `docs/CONTRIBUTING.md` -- Contribution guide

## Constraints

- Use `thiserror` for typed errors in oops-core, per-command error enums in oops-cli
- Each command implements the `Op` trait with typed `Error` and `Output`
- Shared rendering lives in `ui/` module; per-command rendering in `commands/<cmd>/render.rs`
- Use `disk_size()` (block-level via `stat.blocks * 512`) not `metadata.len()`
- Status messages go to stderr, machine-readable output to stdout
- CLI binary is named `oops`

## Do Not

- Push directly to main
- Skip CI checks with --no-verify
- Add new dependencies without justification
- Output ANSI color codes to stdout
- Put shared rendering logic in command files (it belongs in ui/)
