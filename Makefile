.PHONY: check install wiki profile bench

PORT ?= 4000
# Directory to scan for `make profile` / `make bench`
DIR ?= $(HOME)
# Expand a leading ~ (make passes `DIR=~/x` through literally)
SCAN_DIR = $(patsubst ~%,$(HOME)%,$(DIR))

check:
	cargo build
	cargo test
	cargo clippy -- -D warnings
	cargo fmt --check

install:
	cargo install --path crates/oops-cli

wiki:
	cd wiki && bundle exec jekyll serve --livereload --port $(PORT)

# Record a CPU profile of a scan and open it in the Firefox Profiler (needs `samply`)
profile:
	cargo build --profile profiling
	samply record target/profiling/oops $(SCAN_DIR)

# Time a scan with warmup runs (needs `hyperfine`)
bench:
	cargo build --release
	hyperfine --warmup 2 --runs 10 --shell=none 'target/release/oops $(SCAN_DIR)'
