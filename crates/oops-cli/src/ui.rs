//! Shared rendering utilities for CLI output.
//!
//! This module owns ALL visual output: status messages, color helpers,
//! bar rendering, table construction, and formatting. Commands call into
//! here rather than building tables or formatting directly.

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bytesize::ByteSize;
use colored::Colorize;
use comfy_table::{presets, Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

use oops_core::{DirEntry, Volume, WasteCategory, WasteEntry};

use crate::commands::drill::{DrillLevel, StopReason};

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
// Symbols
// ---------------------------------------------------------------------------

pub const SYM_OK: &str = "\u{2713}";
pub const SYM_BLOCK: &str = "\u{2588}";
pub const SYM_BLOCK_MED: &str = "\u{2593}";
pub const SYM_BLOCK_LIGHT: &str = "\u{2591}";

// ---------------------------------------------------------------------------
// Status helpers (stderr)
// ---------------------------------------------------------------------------

pub fn success(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{} {}", SYM_OK.green(), msg);
    }
}

pub fn header(msg: &str) {
    if is_plain() {
        eprintln!("{}", msg);
    } else {
        eprintln!("{}", msg.bold());
    }
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

pub fn highlight(s: &str) -> String {
    if is_plain() { s.to_string() } else { s.cyan().to_string() }
}

pub fn bold(s: &str) -> String {
    if is_plain() { s.to_string() } else { s.bold().to_string() }
}

pub fn dim(s: &str) -> String {
    if is_plain() { s.to_string() } else { s.dimmed().to_string() }
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

enum BarColor { Green, Yellow, Red }

fn capacity_color(fraction: f64) -> BarColor {
    if fraction >= 0.90 { BarColor::Red }
    else if fraction >= 0.70 { BarColor::Yellow }
    else { BarColor::Green }
}

fn capacity_cell_color(fraction: f64) -> Color {
    if fraction >= 0.90 { Color::Red }
    else if fraction >= 0.70 { Color::Yellow }
    else { Color::Green }
}

/// Render a usage bar: [########--------] 65%
pub fn usage_bar(fraction: f64, width: usize) -> String {
    let filled = (fraction * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    if is_plain() {
        return format!("[{}{}] {:>3.0}%", "#".repeat(filled), "-".repeat(empty), fraction * 100.0);
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

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s.char_indices().nth(max - 1).map(|(i, _)| i).unwrap_or(s.len());
        format!("{}\u{2026}", &s[..end])
    }
}

// ---------------------------------------------------------------------------
// Error display
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Table builders — all table construction lives here
// ---------------------------------------------------------------------------

/// Render the volumes table to stderr.
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
        let bar = usage_bar(fraction, 20);

        table.add_row(vec![
            Cell::new(vol.mount_point.display().to_string()).fg(Color::White),
            Cell::new(truncate(&vol.filesystem, 24)).fg(Color::DarkGrey),
            Cell::new(fmt_size(vol.total)).set_alignment(CellAlignment::Right),
            Cell::new(fmt_size(vol.used)).fg(color).set_alignment(CellAlignment::Right),
            Cell::new(fmt_size(vol.available)).set_alignment(CellAlignment::Right),
            Cell::new(bar),
        ]);
    }

    eprintln!("{table}");
}

/// Render directory entries as a proportional breakdown to stderr.
pub fn render_dir_breakdown(path: &std::path::Path, entries: &[DirEntry], total_size: u64) {
    eprintln!(
        "{} {}",
        bold(&short_path(path)),
        dim(&format!("({})", fmt_size(total_size))),
    );
    eprintln!();

    if entries.is_empty() {
        eprintln!("  (empty directory)");
        return;
    }

    let bar_width = 24;
    let max_name_len = entries.iter().take(20).map(|e| e.name.len() + 1).max().unwrap_or(10).min(40);

    for entry in entries.iter().take(20) {
        let fraction = if total_size > 0 { entry.size as f64 / total_size as f64 } else { 0.0 };
        let bar = proportion_bar(fraction, bar_width);
        let name = if entry.is_dir { format!("{}/", entry.name) } else { entry.name.clone() };
        let name_display = truncate(&name, max_name_len);
        let size_str = fmt_size(entry.size);
        let pct_str = format!("{:>5.1}%", fraction * 100.0);

        if is_plain() {
            eprintln!("  {:<width$}  {:>10}  {}", name_display, size_str, pct_str, width = max_name_len);
        } else {
            eprintln!(
                "  {:<width$}  {:>10}  {}  {}",
                name_display, size_str, dim(&pct_str), bar,
                width = max_name_len,
            );
        }
    }

    if entries.len() > 20 {
        let rest_size: u64 = entries.iter().skip(20).map(|e| e.size).sum();
        eprintln!("  {} ({} more items, {})", dim("..."), entries.len() - 20, fmt_size(rest_size));
    }

}

/// Render the top-N largest entries table to stderr.
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
        let type_color = if entry.is_dir { Color::Cyan } else { Color::White };
        let rel_path = entry.path.strip_prefix(base_path).unwrap_or(&entry.path);

        table.add_row(vec![
            Cell::new(i + 1).fg(Color::DarkGrey).set_alignment(CellAlignment::Right),
            Cell::new(fmt_size(entry.size)).set_alignment(CellAlignment::Right),
            Cell::new(type_str).fg(type_color),
            Cell::new(truncate(&rel_path.display().to_string(), 72)).fg(Color::White),
        ]);
    }

    eprintln!("{table}");

    let total: u64 = entries.iter().map(|e| e.size).sum();
    eprintln!();
    eprintln!("  {} {} across {} entries", bold("Total:"), highlight(&fmt_size(total)), entries.len());
}

/// Render a tree node (recursive, called by the tree command).
pub fn render_tree_node(
    entries: &[DirEntry],
    parent_total: u64,
    depth: usize,
    min_pct: f64,
) {
    let indent = "  ".repeat(depth + 1);
    let connector_mid = "\u{251c}\u{2500}\u{2500}";
    let connector_end = "\u{2514}\u{2500}\u{2500}";

    let shown: Vec<_> = entries
        .iter()
        .filter(|e| {
            parent_total > 0 && (e.size as f64 / parent_total as f64) * 100.0 >= min_pct
        })
        .collect();

    let hidden_count = entries.len() - shown.len();
    let hidden_size: u64 = entries
        .iter()
        .filter(|e| parent_total == 0 || (e.size as f64 / parent_total as f64) * 100.0 < min_pct)
        .map(|e| e.size)
        .sum();

    for (i, entry) in shown.iter().enumerate() {
        let is_last = i == shown.len() - 1 && hidden_count == 0;
        let connector = if is_last { connector_end } else { connector_mid };

        let pct = if parent_total > 0 {
            (entry.size as f64 / parent_total as f64) * 100.0
        } else {
            0.0
        };

        let bar = proportion_bar(pct / 100.0, 16);
        let name = if entry.is_dir { format!("{}/", entry.name) } else { entry.name.clone() };

        if is_plain() {
            eprintln!("{}{} {:>10}  {:>5.1}%  {}", indent, connector, fmt_size(entry.size), pct, name);
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
            indent, dim(connector_end), fmt_size(hidden_size), "", dim("..."), hidden_count, min_pct,
        );
    }
}

/// Render sweep results: summary by category + optional detail table.
pub fn render_sweep_results(entries: &[WasteEntry], verbose: bool) {
    if entries.is_empty() {
        success("No significant waste found.");
        return;
    }

    // Group by category
    let mut by_category: HashMap<String, (WasteCategory, u64, usize)> = HashMap::new();
    for entry in entries {
        let key = entry.category.label().to_string();
        let e = by_category.entry(key).or_insert_with(|| (entry.category.clone(), 0, 0));
        e.1 += entry.size;
        e.2 += 1;
    }

    let mut categories: Vec<_> = by_category.into_values().collect();
    categories.sort_by(|a, b| b.1.cmp(&a.1));

    header("Reclaimable Space by Category");
    eprintln!();

    let mut table = Table::new();
    table
        .load_preset(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("CATEGORY").add_attribute(Attribute::Bold),
            Cell::new("SIZE").add_attribute(Attribute::Bold),
            Cell::new("COUNT").add_attribute(Attribute::Bold),
            Cell::new("DESCRIPTION").add_attribute(Attribute::Bold),
        ]);

    let total_waste: u64 = categories.iter().map(|(_, size, _)| size).sum();

    for (cat, size, count) in &categories {
        table.add_row(vec![
            Cell::new(cat.label()).fg(Color::Cyan),
            Cell::new(fmt_size(*size)).fg(Color::Yellow).set_alignment(CellAlignment::Right),
            Cell::new(count).set_alignment(CellAlignment::Right),
            Cell::new(cat.description()).fg(Color::DarkGrey),
        ]);
    }

    eprintln!("{table}");
    eprintln!();
    eprintln!("  {} {}", bold("Total reclaimable:"), highlight(&fmt_size(total_waste)));

    if verbose {
        eprintln!();
        header("Individual Entries");
        eprintln!();

        let mut detail_table = Table::new();
        detail_table
            .load_preset(presets::NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("SIZE").add_attribute(Attribute::Bold),
                Cell::new("CATEGORY").add_attribute(Attribute::Bold),
                Cell::new("PATH").add_attribute(Attribute::Bold),
            ]);

        for entry in entries.iter().take(50) {
            detail_table.add_row(vec![
                Cell::new(fmt_size(entry.size)).fg(Color::Yellow).set_alignment(CellAlignment::Right),
                Cell::new(entry.category.label()).fg(Color::Cyan),
                Cell::new(short_path(&entry.path)).fg(Color::White),
            ]);
        }

        eprintln!("{detail_table}");

        if entries.len() > 50 {
            eprintln!();
            eprintln!("  {} ({} more entries not shown)", dim("..."), entries.len() - 50);
        }
    }
}

// ---------------------------------------------------------------------------
// Drill rendering
// ---------------------------------------------------------------------------

// Box-drawing characters for tree rendering
const PIPE: &str = "\u{2502}";       // │
const TEE: &str = "\u{251c}\u{2500}"; // ├─
const ELL: &str = "\u{2514}\u{2500}"; // └─
const ARROW_DOWN: &str = "\u{25bc}";  // ▼

/// Render a single drill level. `depth` is 0-based. `has_next` indicates
/// whether the drill will continue (controls the ▼ marker on the biggest dir).
pub fn render_drill_level(level: &DrillLevel, depth: usize, has_next: bool) {
    let prefix = build_prefix(depth);

    // Directory header
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
        eprintln!(
            "{}{} {} {}",
            prefix,
            if is_plain() { ARROW_DOWN } else { &ARROW_DOWN.cyan().to_string() },
            bold(&level.dir_name),
            dim(&format!("({})", size_str)),
        );
    }

    if level.total == 0 || level.entries.is_empty() {
        return;
    }

    // Render children with tree connectors
    let show_count = level.show_count.min(level.entries.len());
    let remaining = level.entries.len().saturating_sub(show_count);
    let has_more = remaining > 0;

    // Find the drilled-into entry (first dir, which is biggest since sorted)
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
        let bar = proportion_bar(frac, 16);

        let name = if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };

        // Highlight the drilled entry
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
                prefix, connector, fmt_size(entry.size), pct, name_display, marker,
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
                prefix, ELL, fmt_size(rest_size), "", remaining,
            );
        } else {
            eprintln!(
                "{}{} {:>10}          {:>16}  {} ({} more)",
                prefix, dim(ELL), fmt_size(rest_size), "", dim("..."), remaining,
            );
        }
    }

    // Stop reason
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

/// Render the trail summary at the end of a drill.
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

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A terminal spinner that renders below existing output on stderr.
/// Clears itself when stopped or dropped. No-ops in plain mode.
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(msg: &str) -> Self {
        use std::io::IsTerminal;
        let running = Arc::new(AtomicBool::new(true));

        if is_plain() || !std::io::stderr().is_terminal() {
            return Spinner { running, handle: None };
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
            // Clear the spinner line
            let _ = write!(stderr, "\r\x1b[2K");
            let _ = stderr.flush();
        });

        Spinner {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(self) {
        // drop triggers cleanup
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
