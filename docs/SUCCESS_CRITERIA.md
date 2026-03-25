# Success Criteria

Checks that must pass before code can be merged. This is the CI gate.

## Quick Check

```bash
make check
```

Or run individually:
```bash
cargo build && cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

## Individual Checks

### Build

```bash
cargo build
```

Build must complete without errors.

### Tests

```bash
cargo test
```

All tests must pass.

### Linting

```bash
cargo clippy -- -D warnings
```

No clippy warnings allowed. Common issues:
- Unused variables or imports
- Unnecessary clones
- Missing error handling

### Formatting

Check formatting:
```bash
cargo fmt --check
```

Fix formatting:
```bash
cargo fmt
```

## Fixing Common Issues

### Formatting Failures

```bash
cargo fmt
git add -p
git commit -m "style: format code"
```

### Lint Warnings

Fix the warning in code, or if it's a false positive:
```rust
#[allow(clippy::lint_name)]
```

Only suppress lints with good reason.

### Test Failures

Run a specific test with output:
```bash
cargo test test_name -- --nocapture
```
