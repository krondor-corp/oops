# Contributing

Guide for both human contributors and AI agents working on oops.

## Getting Started

1. Clone the repository
   ```bash
   git clone https://github.com/krondor-corp/oops
   cd oops
   ```

2. Build the project
   ```bash
   cargo build
   ```

3. Run tests
   ```bash
   cargo test
   ```

4. Install locally
   ```bash
   make install
   ```

## Making Changes

1. Create a feature branch from `main`
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make your changes following the patterns in `docs/PATTERNS.md`

3. Run all checks
   ```bash
   make check
   ```

4. Commit with a clear message describing the change

5. Open a pull request

## Profiling

Scanning is dominated by filesystem syscalls, so measure before optimizing.
Install the tools with `brew install samply hyperfine` (or `cargo install samply`
and your distro's `hyperfine` on Linux).

```bash
make profile                    # flamegraph of `oops ~` in the Firefox Profiler
make profile DIR=~/Library      # profile a specific directory
make bench DIR=~/repos          # wall-clock timing over 10 runs
RAYON_NUM_THREADS=4 make bench  # compare thread counts
```

`make profile` builds with the `profiling` Cargo profile (release + debug symbols).
In the profiler, the "Flame Graph" tab gives the classic view; use "Invert call
stack" in the Call Tree to see which syscalls dominate.

Note that results depend heavily on whether the scanned tree fits in the
kernel's vnode cache (`sysctl kern.maxvnodes` on macOS): small trees are
CPU/lock bound, trees larger than the cache are I/O bound.

## Commit Message Format

Use conventional commits:
- `feat:` -- New feature
- `fix:` -- Bug fix
- `docs:` -- Documentation only
- `style:` -- Formatting, no code change
- `refactor:` -- Code restructuring
- `test:` -- Adding or updating tests
- `chore:` -- Maintenance tasks

The release automation keys off these prefixes: `feat` triggers a minor bump, `fix` a patch, and `feat!` / `BREAKING CHANGE` a major. Merges with neither produce no release PR.

Examples:
```
feat: add --min-size flag to top command
fix: report block-level sizes for sparse files
docs: add Docker disk usage recipe to wiki
```

## Releases

Releases are fully automated off `main`:

1. **`release-pr.yml`** -- on every push to `main`, scans commits since the last tag for `feat` / `fix` / `feat!`. If any are found, it (re)opens a `release-automation` PR that bumps the workspace version in the three `Cargo.toml` files.
2. **`release-tag.yml`** -- when that PR merges, a commit containing `release-automation` lands on `main`. The workflow reads the bumped version, creates `vX.Y.Z`, and pushes the tag using `secrets.RELEASE_PAT` (a PAT is required so the tag push can trigger the next workflow -- `GITHUB_TOKEN` cannot).
3. **`release.yml`** -- triggered by the `v*` tag. Builds `oops` for `aarch64-darwin` and `x86_64-linux`, packages each as `oops-vX.Y.Z-<arch>-<os>.tar.gz`, and publishes a GitHub Release with the artifacts attached.
4. **`install.sh`** at repo root pulls those artifacts for the user's platform. `oops update` re-runs the same script in place.

Adding a new release target means extending the matrix in `.github/workflows/release.yml` and the arch/os case in `install.sh` together.

## For AI Agents

### Context to Gather First

Before making changes, read:
- `CLAUDE.md` -- Project overview and quick commands
- `docs/PATTERNS.md` -- Coding conventions
- `docs/SUCCESS_CRITERIA.md` -- CI checks that must pass
- Related code files to understand existing patterns

### Constraints

- Don't modify CI/CD configuration without approval
- Don't add new dependencies without discussion
- Don't refactor unrelated code
- Don't skip tests or use --no-verify
- Always run `cargo fmt` before committing
- Keep stdout free of color codes
- Rendering logic belongs in `ui.rs`, not in command files

## Code Review

- All PRs require review before merge
- CI must pass (build, test, clippy, fmt)
- Squash commits on merge when appropriate
