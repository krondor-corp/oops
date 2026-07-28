//! Shared rendering utilities for CLI output.
//!
//! Provides consistent status symbols, color helpers, truncation,
//! and a global plain-mode flag for scriptable output.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bytesize::ByteSize;
use colored::Colorize;

use oops_core::Volume;

pub mod colors;
pub mod output;

pub use colors::*;
pub use output::*;

// ---------------------------------------------------------------------------
// Plain mode
// ---------------------------------------------------------------------------

static PLAIN_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_plain(enabled: bool) {
    PLAIN_MODE.store(enabled, Ordering::Relaxed);
}

pub fn is_plain() -> bool {
    PLAIN_MODE.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Terminal width
// ---------------------------------------------------------------------------

/// Current terminal width (columns), with a sane fallback.
pub fn term_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

/// Compute a bar width that scales to terminal size.
/// Returns roughly `fraction` of the terminal width, clamped to [min, max].
pub fn scaled_bar(fraction: f64, min: usize, max: usize) -> usize {
    let w = (term_width() as f64 * fraction).round() as usize;
    w.clamp(min, max)
}

// ---------------------------------------------------------------------------
// Size formatting
// ---------------------------------------------------------------------------

pub fn fmt_size(bytes: u64) -> String {
    ByteSize(bytes).to_string()
}

// ---------------------------------------------------------------------------
// Bar rendering
// ---------------------------------------------------------------------------

pub const SYM_BLOCK: &str = "\u{2588}";
pub const SYM_BLOCK_LIGHT: &str = "\u{2591}";

enum BarColor {
    Green,
    Yellow,
    Red,
}

fn capacity_color(fraction: f64) -> BarColor {
    if fraction >= 0.90 {
        BarColor::Red
    } else if fraction >= 0.70 {
        BarColor::Yellow
    } else {
        BarColor::Green
    }
}

/// Render a usage bar: [########--------] 65%
pub fn usage_bar(fraction: f64, width: usize) -> String {
    let filled = (fraction * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    if is_plain() {
        return format!(
            "[{}{}] {:>3.0}%",
            "#".repeat(filled),
            "-".repeat(empty),
            fraction * 100.0
        );
    }

    let color = capacity_color(fraction);
    let filled_str = SYM_BLOCK.repeat(filled);
    let empty_str = SYM_BLOCK_LIGHT.repeat(empty);

    let colored_bar = match color {
        BarColor::Green => filled_str.green().to_string(),
        BarColor::Yellow => filled_str.yellow().to_string(),
        BarColor::Red => filled_str.red().to_string(),
    };

    let pct_str = format!("{:>3.0}%", fraction * 100.0);
    let colored_pct = match color {
        BarColor::Green => pct_str.green().to_string(),
        BarColor::Yellow => pct_str.yellow().to_string(),
        BarColor::Red => pct_str.red().bold().to_string(),
    };

    format!("{}{} {}", colored_bar, empty_str.dimmed(), colored_pct)
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

pub fn short_path(path: &std::path::Path) -> String {
    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::Path::new(&home);
        if let Ok(relative) = path.strip_prefix(home_path) {
            return format!("~/{}", relative.display());
        }
    }
    path.display().to_string()
}

// ---------------------------------------------------------------------------
// Truncation
// ---------------------------------------------------------------------------

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max - 1)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}\u{2026}", &s[..end])
    }
}

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(msg: &str) -> Self {
        use std::io::IsTerminal;
        let running = Arc::new(AtomicBool::new(true));

        if is_plain() || !std::io::stderr().is_terminal() {
            return Spinner {
                running,
                handle: None,
            };
        }

        let r = Arc::clone(&running);
        let msg = msg.to_string();
        let handle = std::thread::spawn(move || {
            let mut i = 0;
            let mut stderr = std::io::stderr();
            while r.load(Ordering::Relaxed) {
                let frame = SPINNER_FRAMES[i % SPINNER_FRAMES.len()];
                let _ = write!(stderr, "\r{} {}", frame.cyan(), msg.dimmed());
                let _ = stderr.flush();
                i += 1;
                std::thread::sleep(std::time::Duration::from_millis(80));
            }
            let _ = write!(stderr, "\r\x1b[2K");
            let _ = stderr.flush();
        });

        Spinner {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

// ---------------------------------------------------------------------------
// Command-specific renderers (legacy — move to per-command render.rs over time)
// ---------------------------------------------------------------------------

pub fn render_free(volume: &Volume) {
    let fraction = if volume.total > 0 {
        1.0 - (volume.available as f64 / volume.total as f64)
    } else {
        0.0
    };
    let mount = volume.mount_point.display().to_string();
    let free = fmt_size(volume.available);
    let total = fmt_size(volume.total);
    let bar = usage_bar(fraction, scaled_bar(0.3, 16, 60));

    eprintln!();
    eprintln!(
        "  {} free of {} on {}",
        bold(&highlight(&free)),
        total,
        highlight(&mount),
    );
    eprintln!("  {}", bar);
    eprintln!();
}
