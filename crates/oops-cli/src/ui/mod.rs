//! Shared rendering utilities for CLI output.
//!
//! Provides consistent status symbols, color helpers, truncation, table builders,
//! and a global plain-mode flag for scriptable output.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bytesize::ByteSize;
use colored::Colorize;
use comfy_table::{presets, Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

use oops_core::{DirEntry, Volume};

use crate::commands::drill::{DrillLevel, StopReason};

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
pub const SYM_BLOCK_MED: &str = "\u{2593}";
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

fn capacity_cell_color(fraction: f64) -> Color {
    if fraction >= 0.90 {
        Color::Red
    } else if fraction >= 0.70 {
        Color::Yellow
    } else {
        Color::Green
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

/// Render a proportional bar segment for size visualization.
pub fn proportion_bar(fraction: f64, width: usize) -> String {
    let filled = (fraction * width as f64).round().max(0.0).min(width as f64) as usize;
    let empty = width - filled;
    if is_plain() {
        format!("{}{}", "#".repeat(filled), " ".repeat(empty))
    } else {
        format!(
            "{}{}",
            SYM_BLOCK_MED.repeat(filled).cyan(),
            " ".repeat(empty),
        )
    }
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
// Table helpers
// ---------------------------------------------------------------------------

/// Create a new table with the standard preset (no borders) and dynamic arrangement.
pub fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(
            headers
                .iter()
                .map(|h| Cell::new(*h).add_attribute(Attribute::Bold))
                .collect::<Vec<_>>(),
        );
    table
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

pub fn render_volumes(volumes: &[Volume]) {
    if volumes.is_empty() {
        eprintln!("No volumes found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("MOUNT").add_attribute(Attribute::Bold),
            Cell::new("FILESYSTEM").add_attribute(Attribute::Bold),
            Cell::new("SIZE").add_attribute(Attribute::Bold),
            Cell::new("USED").add_attribute(Attribute::Bold),
            Cell::new("FREE").add_attribute(Attribute::Bold),
            Cell::new("").add_attribute(Attribute::Bold),
        ]);

    for vol in volumes {
        let fraction = vol.capacity_pct / 100.0;
        let color = capacity_cell_color(fraction);
        let bar = usage_bar(fraction, scaled_bar(0.25, 12, 50));

        table.add_row(vec![
            Cell::new(vol.mount_point.display().to_string()).fg(Color::White),
            Cell::new(truncate(&vol.filesystem, scaled_bar(0.15, 16, 40))).fg(Color::DarkGrey),
            Cell::new(fmt_size(vol.total)).set_alignment(CellAlignment::Right),
            Cell::new(fmt_size(vol.used))
                .fg(color)
                .set_alignment(CellAlignment::Right),
            Cell::new(fmt_size(vol.available)).set_alignment(CellAlignment::Right),
            Cell::new(bar),
        ]);
    }

    eprintln!("{table}");
}

pub fn render_dir_breakdown(
    path: &std::path::Path,
    entries: &[DirEntry],
    total_size: u64,
    disk_total: Option<u64>,
) {
    let size_label = match disk_total {
        Some(dt) => format!("({} / {})", fmt_size(total_size), fmt_size(dt)),
        None => format!("({})", fmt_size(total_size)),
    };
    eprintln!("{} {}", bold(&short_path(path)), dim(&size_label),);
    eprintln!();

    if entries.is_empty() {
        eprintln!("  (empty directory)");
        return;
    }

    let tw = term_width();
    let bar_width = scaled_bar(0.3, 16, 60);
    let max_name_len = entries
        .iter()
        .take(20)
        .map(|e| e.name.len() + 1)
        .max()
        .unwrap_or(10)
        .min(tw / 3);

    for entry in entries.iter().take(20) {
        let fraction = if total_size > 0 {
            entry.size as f64 / total_size as f64
        } else {
            0.0
        };
        let bar = proportion_bar(fraction, bar_width);
        let name = if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };
        let name_display = truncate(&name, max_name_len);
        let size_str = fmt_size(entry.size);
        let pct_str = format!("{:>5.1}%", fraction * 100.0);

        if is_plain() {
            eprintln!(
                "  {:<width$}  {:>10}  {}",
                name_display,
                size_str,
                pct_str,
                width = max_name_len
            );
        } else {
            eprintln!(
                "  {:<width$}  {:>10}  {}  {}",
                name_display,
                size_str,
                dim(&pct_str),
                bar,
                width = max_name_len,
            );
        }
    }

    if entries.len() > 20 {
        let rest_size: u64 = entries.iter().skip(20).map(|e| e.size).sum();
        eprintln!(
            "  {} ({} more items, {})",
            dim("..."),
            entries.len() - 20,
            fmt_size(rest_size)
        );
    }
}

pub fn render_top_entries(entries: &[DirEntry], base_path: &std::path::Path) {
    if entries.is_empty() {
        eprintln!("No entries found matching criteria.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#").add_attribute(Attribute::Bold),
            Cell::new("SIZE").add_attribute(Attribute::Bold),
            Cell::new("TYPE").add_attribute(Attribute::Bold),
            Cell::new("PATH").add_attribute(Attribute::Bold),
        ]);

    for (i, entry) in entries.iter().enumerate() {
        let type_str = if entry.is_dir { "dir" } else { "file" };
        let type_color = if entry.is_dir {
            Color::Cyan
        } else {
            Color::White
        };
        let rel_path = entry.path.strip_prefix(base_path).unwrap_or(&entry.path);

        table.add_row(vec![
            Cell::new(i + 1)
                .fg(Color::DarkGrey)
                .set_alignment(CellAlignment::Right),
            Cell::new(fmt_size(entry.size)).set_alignment(CellAlignment::Right),
            Cell::new(type_str).fg(type_color),
            Cell::new(truncate(
                &rel_path.display().to_string(),
                term_width().saturating_sub(25),
            ))
            .fg(Color::White),
        ]);
    }

    eprintln!("{table}");

    let total: u64 = entries.iter().map(|e| e.size).sum();
    eprintln!();
    eprintln!(
        "  {} {} across {} entries",
        bold("Total:"),
        highlight(&fmt_size(total)),
        entries.len()
    );
}

pub fn render_tree_node(entries: &[DirEntry], parent_total: u64, depth: usize, min_pct: f64) {
    let indent = "  ".repeat(depth + 1);
    let connector_mid = "\u{251c}\u{2500}\u{2500}";
    let connector_end = "\u{2514}\u{2500}\u{2500}";

    let shown: Vec<_> = entries
        .iter()
        .filter(|e| parent_total > 0 && (e.size as f64 / parent_total as f64) * 100.0 >= min_pct)
        .collect();

    let hidden_count = entries.len() - shown.len();
    let hidden_size: u64 = entries
        .iter()
        .filter(|e| parent_total == 0 || (e.size as f64 / parent_total as f64) * 100.0 < min_pct)
        .map(|e| e.size)
        .sum();

    for (i, entry) in shown.iter().enumerate() {
        let is_last = i == shown.len() - 1 && hidden_count == 0;
        let connector = if is_last {
            connector_end
        } else {
            connector_mid
        };

        let pct = if parent_total > 0 {
            (entry.size as f64 / parent_total as f64) * 100.0
        } else {
            0.0
        };

        let bar = proportion_bar(pct / 100.0, scaled_bar(0.2, 10, 40));
        let name = if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };

        if is_plain() {
            eprintln!(
                "{}{} {:>10}  {:>5.1}%  {}",
                indent,
                connector,
                fmt_size(entry.size),
                pct,
                name
            );
        } else {
            eprintln!(
                "{}{} {:>10}  {}  {}  {}",
                indent,
                dim(connector),
                fmt_size(entry.size),
                dim(&format!("{:>5.1}%", pct)),
                bar,
                if entry.is_dir { bold(&name) } else { name },
            );
        }
    }

    if hidden_count > 0 && hidden_size > 0 {
        eprintln!(
            "{}{} {:>10}          {:>16}  {} ({} items below {:.1}%)",
            indent,
            dim(connector_end),
            fmt_size(hidden_size),
            "",
            dim("..."),
            hidden_count,
            min_pct,
        );
    }
}

// ---------------------------------------------------------------------------
// Drill rendering
// ---------------------------------------------------------------------------

const PIPE: &str = "\u{2502}";
const TEE: &str = "\u{251c}\u{2500}";
const ELL: &str = "\u{2514}\u{2500}";
const ARROW_DOWN: &str = "\u{25bc}";

pub fn render_drill_level(level: &DrillLevel, depth: usize, has_next: bool) {
    let prefix = build_prefix(depth);

    let size_str = if level.total > 0 {
        fmt_size(level.total)
    } else {
        "empty".to_string()
    };

    if depth == 0 {
        eprintln!(
            "{} {}",
            bold(&level.dir_name),
            dim(&format!("({})", size_str)),
        );
    } else {
        let arrow = if is_plain() {
            ARROW_DOWN.to_string()
        } else {
            ARROW_DOWN.cyan().to_string()
        };
        eprintln!(
            "{}{} {} {}",
            prefix,
            arrow,
            bold(&level.dir_name),
            dim(&format!("({})", size_str)),
        );
    }

    if level.total == 0 || level.entries.is_empty() {
        return;
    }

    let show_count = level.show_count.min(level.entries.len());
    let remaining = level.entries.len().saturating_sub(show_count);
    let has_more = remaining > 0;

    let drilled_idx = if has_next {
        level.entries.iter().position(|e| e.is_dir)
    } else {
        None
    };

    for (i, entry) in level.entries.iter().take(show_count).enumerate() {
        let is_last_entry = i == show_count - 1 && !has_more;
        let connector = if is_last_entry { ELL } else { TEE };
        let is_drilled = drilled_idx == Some(i);

        let frac = if level.total > 0 {
            entry.size as f64 / level.total as f64
        } else {
            0.0
        };
        let pct = frac * 100.0;
        let bar = proportion_bar(frac, scaled_bar(0.2, 10, 40));

        let name = if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };

        let name_display = if is_drilled && !is_plain() {
            name.cyan().bold().to_string()
        } else if entry.is_dir {
            bold(&name)
        } else {
            name
        };

        let marker = if is_drilled && !is_plain() {
            format!(" {}", ARROW_DOWN.cyan())
        } else {
            String::new()
        };

        if is_plain() {
            eprintln!(
                "{}{} {:>10}  {:>5.1}%  {}{}",
                prefix,
                connector,
                fmt_size(entry.size),
                pct,
                name_display,
                marker,
            );
        } else {
            eprintln!(
                "{}{} {:>10}  {}  {}  {}{}",
                prefix,
                dim(connector),
                fmt_size(entry.size),
                dim(&format!("{:>5.1}%", pct)),
                bar,
                name_display,
                marker,
            );
        }
    }

    if has_more {
        let rest_size: u64 = level.entries.iter().skip(show_count).map(|e| e.size).sum();
        if is_plain() {
            eprintln!(
                "{}{} {:>10}          {:>16}  ... ({} more)",
                prefix,
                ELL,
                fmt_size(rest_size),
                "",
                remaining,
            );
        } else {
            eprintln!(
                "{}{} {:>10}          {:>16}  {} ({} more)",
                prefix,
                dim(ELL),
                fmt_size(rest_size),
                "",
                dim("..."),
                remaining,
            );
        }
    }

    if let Some(ref reason) = level.stop_reason {
        eprintln!();
        match reason {
            StopReason::BelowThreshold { pct, threshold } => {
                eprintln!(
                    "{}{}",
                    prefix,
                    dim(&format!(
                        "stopped: largest child is {:.1}% (below {:.0}% threshold)",
                        pct, threshold,
                    )),
                );
            }
            StopReason::NoSubdirectories => {
                eprintln!("{}{}", prefix, dim("(no subdirectories)"));
            }
            StopReason::Empty => {}
        }
    }
}

pub fn render_drill_trail(names: &[String]) {
    eprintln!();
    let trail: Vec<&str> = names.iter().map(|n| n.trim_end_matches('/')).collect();
    eprintln!("{} {}", bold("Trail:"), trail.join(" > "));
}

fn build_prefix(depth: usize) -> String {
    if depth == 0 {
        return String::new();
    }
    if is_plain() {
        "   ".repeat(depth)
    } else {
        let segment = format!("{}  ", dim(PIPE));
        segment.repeat(depth)
    }
}
