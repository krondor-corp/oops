---
description: Push current branch and create a draft PR. Use when ready to share work for review or collaborate on a branch.
allowed-tools:
  - Bash(git:*)
  - Bash(gh pr:*)
  - Bash(gh repo:*)
  - Bash(cargo:*)
  - Bash(make:*)
  - Read
  - Glob
  - Grep
---

Create a draft pull request for the current branch.

## Steps

1. Get the current branch name:
   ```
   git branch --show-current
   ```

2. Check for uncommitted changes:
   ```
   git status --porcelain
   ```
   If there are uncommitted changes:
   a. Run `cargo fmt` to fix formatting
   b. Stage all changes: `git add -A`
   c. Create a commit with a descriptive message using conventional commits

3. Check if the branch has an upstream:
   ```
   git status -sb
   ```
   - If no upstream: `git push -u origin <branch>`
   - If upstream exists: `git push`

4. Determine the base branch (check for `main`, then `master`, then default).

5. Gather context -- commits unique to this branch:
   ```
   git log <base>..HEAD --oneline
   ```

6. Create a draft PR:
   ```
   gh pr create --draft --base <base>
   ```
   - Title: descriptive of what the branch accomplishes
   - Body: summarize ALL changes based on the commits

7. Return the PR URL to the user.

## Important

- Commit ALL uncommitted changes before pushing
- Do NOT use `--no-verify` when pushing
- Run `cargo fmt` before committing
