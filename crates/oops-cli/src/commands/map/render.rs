//! Proportional tree renderer for the map command.
//!
//! Each entry gets its own line: a proportional bar, size, and indented name.
//! The bar width encodes size relative to the root — your eye immediately
//! finds the biggest entries. Hierarchy is shown via indentation.

use colored::Colorize;

use oops_core::{TreeNode, Volume};

use crate::ui;

const BAR_CHAR: &str = "\u{2501}"; // ━
const BAR_CHAR_PLAIN: &str = "=";

const DEPTH_COLORS: &[fn(&str) -> String] = &[
    |s: &str| s.cyan().to_string(),
    |s: &str| s.green().to_string(),
    |s: &str| s.yellow().to_string(),
    |s: &str| s.magenta().to_string(),
    |s: &str| s.blue().to_string(),
    |s: &str| s.red().to_string(),
];

struct Row {
    name: String,
    size: u64,
    depth: usize,
    is_dir: bool,
    is_aggregate: bool,
}

pub fn render(
    root: &TreeNode,
    volume: Option<&Volume>,
    term_width: usize,
    max_depth: usize,
    show: usize,
    expand: usize,
) {
    if root.size == 0 {
        eprintln!("  (empty or inaccessible)");
        return;
    }

    // Volume context
    if let Some(vol) = volume {
        render_disk_context(vol, root, term_width);
        eprintln!();
    }

    // Build rows
    let rows = build_rows(root, max_depth, show, expand);

    // Layout: 2 margin + bar + 2 gap + 10 size + 2 gap + indent+name
    let bar_width = ((term_width as f64) * 0.38).round() as usize;
    let bar_width = bar_width.clamp(12, 60);
    let size_col = 10;

    for row in &rows {
        render_row(row, root.size, bar_width, size_col, term_width);
    }
    eprintln!();
}

// ---------------------------------------------------------------------------
// Row collection
// ---------------------------------------------------------------------------

fn build_rows(root: &TreeNode, max_depth: usize, show: usize, expand: usize) -> Vec<Row> {
    let mut rows = Vec::new();

    // Root row — show `/` instead of `/System/Volumes/Data/` (APFS redirect)
    let short = ui::short_path(&root.path);
    let root_name = if short == "/System/Volumes/Data" || short == "/System/Volumes/Data/" {
        "/".to_string()
    } else if root.is_dir && !short.ends_with('/') {
        format!("{}/", short)
    } else {
        short
    };
    rows.push(Row {
        name: root_name,
        size: root.size,
        depth: 0,
        is_dir: root.is_dir,
        is_aggregate: false,
    });

    collect_children(&root.children, 1, max_depth, show, expand, &mut rows);
    rows
}

fn collect_children(
    children: &[TreeNode],
    depth: usize,
    max_depth: usize,
    show: usize,
    expand: usize,
    rows: &mut Vec<Row>,
) {
    if depth > max_depth || children.is_empty() {
        return;
    }

    let visible_count = show.min(children.len());
    let visible = &children[..visible_count];
    let hidden = &children[visible_count..];

    for (i, child) in visible.iter().enumerate() {
        let name = if child.is_dir {
            format!("{}/", child.name)
        } else {
            child.name.clone()
        };

        rows.push(Row {
            name,
            size: child.size,
            depth,
            is_dir: child.is_dir,
            is_aggregate: false,
        });

        if i < expand && child.is_dir && !child.children.is_empty() {
            collect_children(&child.children, depth + 1, max_depth, show, expand, rows);
        }
    }

    if !hidden.is_empty() {
        let hidden_size: u64 = hidden.iter().map(|c| c.size).sum();
        rows.push(Row {
            name: format!("({} others)", hidden.len()),
            size: hidden_size,
            depth,
            is_dir: false,
            is_aggregate: true,
        });
    }
}

// ---------------------------------------------------------------------------
// Row rendering
// ---------------------------------------------------------------------------

fn render_row(row: &Row, root_size: u64, bar_width: usize, size_col: usize, term_width: usize) {
    let frac = if root_size > 0 {
        row.size as f64 / root_size as f64
    } else {
        0.0
    };
    let filled = (frac * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let indent = "  ".repeat(row.depth);
    let size_str = ui::fmt_size(row.size);

    // Name budget: whatever is left after margin + bar + gaps + size
    let overhead = 2 + bar_width + 2 + size_col + 2;
    let name_budget = term_width.saturating_sub(overhead);
    let full_name = format!("{}{}", indent, row.name);
    let display_name = if full_name.len() > name_budget && name_budget > 4 {
        ui::truncate(&full_name, name_budget)
    } else {
        full_name
    };

    if ui::is_plain() {
        let bar = BAR_CHAR_PLAIN.repeat(filled);
        eprintln!(
            "  {}{} {:>w$}  {}",
            bar,
            " ".repeat(empty),
            size_str,
            display_name,
            w = size_col,
        );
    } else {
        let color_fn = DEPTH_COLORS[row.depth % DEPTH_COLORS.len()];
        let bar = color_fn(&BAR_CHAR.repeat(filled));
        let pad = " ".repeat(empty);

        let name_display = if row.is_aggregate {
            ui::dim(&display_name)
        } else if row.is_dir {
            ui::bold(&display_name)
        } else {
            display_name
        };

        let size_display = color_fn(&format!("{:>w$}", size_str, w = size_col));

        eprintln!("  {}{}  {}  {}", bar, pad, size_display, name_display);
    }
}

// ---------------------------------------------------------------------------
// Disk context
// ---------------------------------------------------------------------------

fn render_disk_context(vol: &Volume, root: &TreeNode, term_width: usize) {
    let total = vol.total;
    if total == 0 {
        return;
    }

    let scanned = root.size;
    let free = vol.available;
    // Scanned can exceed total on APFS (firmlinks cause double counting)
    let used = total.saturating_sub(free);
    let other = used.saturating_sub(scanned);

    eprintln!(
        "  {} {}",
        ui::bold(&vol.mount_point.display().to_string()),
        ui::dim(&format!(
            "({} total, {} free)",
            ui::fmt_size(total),
            ui::fmt_size(free),
        )),
    );

    let is_root_scan = root.path == vol.mount_point || root.path.to_str() == Some("/");
    let other_pct = other as f64 / total as f64;
    if !is_root_scan && other_pct > 0.10 {
        eprintln!(
            "  {}",
            ui::dim(&format!(
                "tip: {} unaccounted for — run 'oops map {}' to explore",
                ui::fmt_size(other),
                vol.mount_point.display(),
            )),
        );
    }

    // Compact context bar
    let bar_w = (term_width as f64 * 0.38).round().clamp(12.0, 60.0) as usize;
    let s_w = (scanned as f64 / total as f64 * bar_w as f64).round() as usize;
    let f_w = (free as f64 / total as f64 * bar_w as f64).round() as usize;
    let o_w = bar_w.saturating_sub(s_w).saturating_sub(f_w);

    if ui::is_plain() {
        eprintln!(
            "  {}{}{}",
            "#".repeat(s_w),
            "=".repeat(o_w),
            "-".repeat(f_w),
        );
    } else {
        eprintln!(
            "  {}{}{}",
            BAR_CHAR.repeat(s_w).cyan(),
            BAR_CHAR.repeat(o_w).dimmed(),
            BAR_CHAR.repeat(f_w).green().dimmed(),
        );
    }

    // Legend
    let legend = format!(
        "scanned {}  other {}  free {}",
        ui::fmt_size(scanned),
        ui::fmt_size(other),
        ui::fmt_size(free),
    );
    eprintln!("  {}", ui::dim(&legend));
}
