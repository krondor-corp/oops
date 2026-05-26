use colored::Colorize;

use super::is_plain;

// ---------------------------------------------------------------------------
// Symbols
// ---------------------------------------------------------------------------

pub const SYM_OK: &str = "\u{2713}";
pub const SYM_ARROW: &str = "\u{2192}";

// ---------------------------------------------------------------------------
// Status helpers (stderr)
// ---------------------------------------------------------------------------

/// Print a success line to stderr: `✓ message`
pub fn success(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{} {}", SYM_OK.green(), msg);
    }
}

/// Print a section header to stderr.
pub fn header(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{}", msg.bold());
    }
}

/// Print a progress line to stderr: `→ message`
pub fn progress(msg: &str) {
    if is_plain() {
        eprintln!("-> {}", msg);
    } else {
        eprintln!("{} {}", SYM_ARROW.cyan(), msg);
    }
}

/// Print an indented detail line to stderr.
pub fn detail(msg: &str) {
    if is_plain() {
        eprintln!("  {}", msg);
    } else {
        eprintln!("  {}", msg.dimmed());
    }
}

/// Print a formatted error chain to stderr.
pub fn print_error(e: &dyn std::error::Error) {
    if is_plain() {
        eprintln!("error: {e}");
    } else {
        eprintln!("{} {e}", "error:".red().bold());
    }
    let mut source = e.source();
    while let Some(cause) = source {
        if is_plain() {
            eprintln!("  caused by: {cause}");
        } else {
            eprintln!("  {} {cause}", "caused by:".yellow());
        }
        source = cause.source();
    }
}
