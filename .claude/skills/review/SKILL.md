---
description: Review branch changes against project conventions. Use when preparing to merge, checking code quality, or validating changes before PR.
allowed-tools:
  - Bash(git diff:*)
  - Bash(git log:*)
  - Bash(git status)
  - Bash(git branch:*)
  - Read
  - Glob
  - Grep
---

Review the current branch's changes against project conventions before merge.

## Steps

### 1. Gather Context

Read project documentation:
- `CLAUDE.md` -- project guide
- `docs/PATTERNS.md` -- coding conventions

### 2. Collect Changes

```
git log main..HEAD --oneline
git diff main...HEAD --stat
git diff main...HEAD
```
If `main` doesn't exist, try `origin/main`.

### 3. Commit Message Audit

```
git log main..HEAD --format="%h %s"
```
Verify they follow conventional commit format.

### 4. Code Review

Review the diff for:
- **Correctness**: Does the logic do what the commit messages claim?
- **Architecture**: Does rendering stay in `ui.rs`? Do commands stay thin?
- **Sizing**: Is `disk_size()` used instead of `metadata.len()`?
- **Error handling**: Typed errors with thiserror?
- **Output**: stderr for UI, stdout for machine-readable only?
- **Dead code**: Leftover debug code, commented-out blocks, unused imports?

### 5. Documentation Check

- `CLAUDE.md` -- Does structure or quick reference need updating?
- `docs/PATTERNS.md` -- Do documented patterns need revision?
- `docs/SUCCESS_CRITERIA.md` -- Did build/test commands change?
- `wiki/` -- Do user-facing docs need updates for changed behavior?

### 6. Skills Check

If behavior changed that affects skills in `.claude/skills/`:
- `/check` -- Did build, test, or lint commands change?
- `/review` -- Did review criteria change?
- `/draft` -- Did PR workflow change?

## Output Format

```
## Commit Messages
- [PASS/FAIL] Format and clarity
- Issues: (list or "None")

## Code Review
- [PASS/WARN/FAIL] Correctness
- [PASS/WARN/FAIL] Architecture (ui.rs owns rendering, commands stay thin)
- [PASS/WARN/FAIL] Sizing (disk_size, not metadata.len)
- [PASS/WARN/FAIL] Error handling
- [PASS/WARN/FAIL] Output conventions
- Suggestions: (list or "None")

## Documentation
- [PASS/WARN] Updates needed: (list or "None")

## Skills
- [PASS/WARN] Updates needed: (list or "None")

## Summary
[Overall assessment and recommended actions before merge]
```

Be specific -- reference file paths and line numbers where relevant.
