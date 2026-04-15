# Documentation Index

Central hub for oops project documentation. AI agents should read this first.

## Quick Start

```bash
cargo build              # Build all crates
cargo test               # Run all tests
cargo clippy             # Lint
cargo fmt --check        # Check formatting
make check               # All of the above
cargo run -- --help      # Run CLI
cargo run --             # Default overview of current directory
cargo run -- drill ~     # Drill into home directory
cargo run -- sweep ~     # Find reclaimable waste
```

## Documentation

| Document | Purpose |
|----------|---------|
| [PATTERNS.md](./PATTERNS.md) | Coding conventions and patterns |
| [CONTRIBUTING.md](./CONTRIBUTING.md) | How to contribute (agents + humans), including the release pipeline |
| [SUCCESS_CRITERIA.md](./SUCCESS_CRITERIA.md) | CI checks that must pass |

## For AI Agents

You are an autonomous coding agent working on a focused task.

### Workflow

1. **Understand** -- Read the task description and relevant docs
2. **Explore** -- Search the codebase to understand context
3. **Plan** -- Break down work into small steps
4. **Implement** -- Follow existing patterns in `PATTERNS.md`
5. **Verify** -- Run checks from `SUCCESS_CRITERIA.md`
6. **Commit** -- Clear, atomic commits

### Guidelines

- Follow existing code patterns and conventions
- Make atomic commits (one logical change per commit)
- Add tests for new functionality
- Update documentation if behavior changes
- If blocked, commit what you have and note the blocker

### When Complete

Your work will be reviewed and merged by the parent session.
Ensure all checks pass before finishing.
