# Coding Patterns

Conventions for contributing to oops.

## Architecture

oops is a Rust workspace with two crates:

- **oops-core** -- Pure library. Scanning and volume detection. No terminal I/O.
- **oops-cli** -- Binary. Commands, UI rendering, terminal output.

## Error Handling

- **oops-core**: Use `thiserror` for typed errors
  - Single `Error` enum in `crates/oops-core/src/lib.rs`
  - Return `Result<T, Error>` from all public functions

- **oops-cli**: Per-command error enums wrapping core errors
  - Each command has its own error type (e.g., `FreeError`, `MapError`)
  - Use `#[error(transparent)]` and `#[from]` for core error conversion
  - Infallible commands use `std::convert::Infallible`

```rust
// In oops-cli command
#[derive(Debug, thiserror::Error)]
pub enum FreeError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Free {
    type Error = FreeError;
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

## UI Module (ui/)

Shared rendering lives in `crates/oops-cli/src/ui/`. Per-command rendering goes in `commands/<cmd>/render.rs` (jig pattern).

- `ui/mod.rs` — plain mode toggle, bar rendering (`usage_bar`), size formatting (`fmt_size`), path helpers (`short_path`), truncation, terminal width, `Spinner`
- `ui/colors.rs` — `bold()`, `dim()`, `highlight()` (gated on plain mode)
- `ui/output.rs` — `success()`, `header()`, `progress()`, `detail()`, `print_error()` (stderr)
- `commands/map/render.rs` — icicle map renderer
- `render_free()` (in `ui/mod.rs`) — free-space summary with usage bar

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
- All shared color/formatting goes through `ui/` helpers, never inline `colored` calls

## Naming Conventions

- **Files/modules**: `snake_case.rs`
- **Types/structs**: `PascalCase`
- **Functions/methods**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **CLI subcommands**: lowercase (free, update)

## Path Handling

- Use `PathBuf` for owned paths, `&Path` for references -- never `String`
- Use `short_path()` from ui to display `~/` prefix for home-relative paths

## Testing

- Unit tests inline with `#[cfg(test)]` modules
- Test pure functions and internal logic
- Run with `cargo test`
