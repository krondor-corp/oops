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
make install             # Install oops binary
make wiki                # Start wiki dev server
```

## Project Structure

```
crates/
├── oops-core/     # Library: scanning, volumes, waste detection
│   ├── scan.rs    # Parallel directory scanning (rayon)
│   ├── volume.rs  # Volume detection via df -Pk
│   ├── sweep.rs   # Waste pattern matching
│   └── lib.rs     # Public API + disk_size() helper
└── oops-cli/      # Binary: commands, UI rendering
    ├── commands/   # One file per subcommand (Op trait)
    ├── ui.rs       # ALL rendering lives here
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
- The `ui` module owns ALL rendering -- commands never format output directly
- Use `disk_size()` (block-level via `stat.blocks * 512`) not `metadata.len()`
- Status messages go to stderr, machine-readable output to stdout
- CLI binary is named `oops`

## Do Not

- Push directly to main
- Skip CI checks with --no-verify
- Add new dependencies without justification
- Output ANSI color codes to stdout
- Put rendering logic in command files (it belongs in ui.rs)
