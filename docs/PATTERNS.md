# Coding Patterns

Conventions for contributing to oops.

## Architecture

oops is a Rust workspace with two crates:

- **oops-core** -- Pure library. Scanning, volumes, waste detection. No terminal I/O.
- **oops-cli** -- Binary. Commands, UI rendering, terminal output.

## Error Handling

- **oops-core**: Use `thiserror` for typed errors
  - Single `Error` enum in `crates/oops-core/src/lib.rs`
  - Return `Result<T, Error>` from all public functions

- **oops-cli**: Per-command error enums wrapping core errors
  - Each command has its own error type (e.g., `DrillError`, `TreeError`)
  - Use `#[error(transparent)]` and `#[from]` for core error conversion
  - Infallible commands use `std::convert::Infallible`

```rust
// In oops-cli command
#[derive(Debug, thiserror::Error)]
pub enum DrillError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Drill {
    type Error = DrillError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        // ...
    }
}
```

## The Op Trait

Every CLI command implements `Op`:

```rust
pub trait Op {
    type Error: std::error::Error;
    type Output;
    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error>;
}
```

- `Ctx` holds the resolved target path and global flags
- `command_enum!` macro generates dispatch from individual command structs
- Doc comments on `Args` structs become CLI help text

## UI Module (ui.rs)

**All rendering lives in `crates/oops-cli/src/ui.rs`.** Commands never format output directly.

- `render_dir_breakdown()` -- Overview proportional bars
- `render_drill_level()` / `render_drill_trail()` -- Drill output
- `render_tree_node()` -- Tree nodes (recursive)
- `render_top_entries()` -- Top-N table
- `render_volumes()` -- Volume capacity bars
- `render_sweep_results()` -- Waste summary + detail
- `proportion_bar()` / `usage_bar()` -- Bar rendering
- `Spinner` -- Loading indicator (TTY-aware, respects plain mode)

Helper functions: `fmt_size()`, `short_path()`, `truncate()`, `bold()`, `dim()`, `highlight()`

## On-Disk Sizing

Always use `disk_size()` from oops-core, never `metadata.len()`:

```rust
fn disk_size(meta: &std::fs::Metadata) -> u64 {
    let blocks = meta.blocks();
    if blocks > 0 { blocks * 512 } else { meta.len() }
}
```

This reports actual block usage. Sparse files (Docker.raw) show real footprint, not inflated apparent size.

## Output Conventions

- **stderr**: All UI output -- status messages, tables, bars, spinners
- **stdout**: Machine-readable output only (reserved for future use)
- **`--plain` flag**: Disables colors and decorations for scripting
- All color/formatting goes through `ui.rs` helpers, never inline `colored` calls

## Naming Conventions

- **Files/modules**: `snake_case.rs`
- **Types/structs**: `PascalCase`
- **Functions/methods**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **CLI subcommands**: lowercase (drill, sweep, tree, top, vol)

## Path Handling

- Use `PathBuf` for owned paths, `&Path` for references -- never `String`
- Use `short_path()` from ui.rs to display `~/` prefix for home-relative paths

## Testing

- Unit tests inline with `#[cfg(test)]` modules
- Test pure functions and internal logic
- Run with `cargo test`
