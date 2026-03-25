---
description: Run project checks (build, test, lint, format). Use when validating code quality, preparing for merge, or verifying changes pass CI.
allowed-tools:
  - Bash(cargo:*)
  - Bash(make:*)
  - Bash(cat:*)
  - Bash(ls:*)
  - Read
  - Glob
  - Grep
---

Run the full success criteria checks for oops.

## Steps

1. Run `make check` which executes:
   - `cargo build`
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`

2. If formatting checks fail, auto-fix with `cargo fmt`, then re-run the check.

3. Report a summary of pass/fail status for each check.

4. If any checks fail that cannot be auto-fixed, report what needs manual attention.

This is the gate for all PRs -- all checks must pass before merge.
