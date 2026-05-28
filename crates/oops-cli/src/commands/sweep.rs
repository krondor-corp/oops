//! Sweep command -- find reclaimable disk space using configurable rules

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

use clap::Args;

use oops_core::{config::OopsConfig, resolve_rules, sweep_directory, SweepMatch};

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Find reclaimable disk space (caches, build artifacts, etc.)
#[derive(Args, Debug, Clone)]
pub struct Sweep {
    /// Target path to scan
    pub path: Option<PathBuf>,

    /// Maximum depth to scan
    #[arg(short, long, default_value = "6")]
    pub depth: usize,

    /// Show individual entries (not just summaries)
    #[arg(short, long)]
    pub verbose: bool,

    /// Interactively confirm and execute cleanup commands
    #[arg(short, long)]
    pub exec: bool,

    /// Filter to specific rule(s) by name
    #[arg(long)]
    pub rule: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum SweepError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Sweep {
    type Error = SweepError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);

        let config = OopsConfig::load()?;
        let mut rules = resolve_rules(&config)?;

        if let Some(ref filter) = self.rule {
            rules.retain(|r| filter.iter().any(|f| r.name.contains(f)));
        }

        let spinner = ui::Spinner::start(&format!(
            "sweeping {} ({} rules)...",
            ui::short_path(target),
            rules.len()
        ));
        let matches = sweep_directory(target, &rules, self.depth)?;
        spinner.stop();

        if self.exec {
            run_interactive(&matches);
        } else {
            render_sweep_results(&matches, self.verbose);
        }

        Ok(NoOutput)
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render_sweep_results(matches: &[SweepMatch], verbose: bool) {
    if matches.is_empty() {
        ui::success("No reclaimable space found.");
        return;
    }

    // Group by rule
    let mut by_rule: Vec<(String, String, u64, usize, bool)> = Vec::new();
    for m in matches {
        if let Some(entry) = by_rule.iter_mut().find(|e| e.0 == m.rule_name) {
            entry.2 += m.size;
            entry.3 += 1;
        } else {
            by_rule.push((
                m.rule_name.clone(),
                m.description.clone(),
                m.size,
                1,
                m.clean_command.is_some(),
            ));
        }
    }
    by_rule.sort_by_key(|e| std::cmp::Reverse(e.2));

    let total: u64 = by_rule.iter().map(|e| e.2).sum();

    ui::header("Reclaimable Space");
    eprintln!();

    let mut table = ui::new_table(&["RULE", "SIZE", "COUNT", "DESCRIPTION"]);
    for (name, desc, size, count, _) in &by_rule {
        use comfy_table::{Cell, CellAlignment, Color};
        table.add_row(vec![
            Cell::new(name).fg(Color::Cyan),
            Cell::new(ui::fmt_size(*size))
                .fg(Color::Yellow)
                .set_alignment(CellAlignment::Right),
            Cell::new(count).set_alignment(CellAlignment::Right),
            Cell::new(desc).fg(Color::DarkGrey),
        ]);
    }
    eprintln!("{table}");
    eprintln!();
    eprintln!(
        "  {} {}",
        ui::bold("Total reclaimable:"),
        ui::highlight(&ui::fmt_size(total)),
    );

    if verbose {
        eprintln!();
        ui::header("Individual Entries");
        eprintln!();
        let mut detail = ui::new_table(&["SIZE", "RULE", "PATH"]);
        for m in matches.iter().take(50) {
            use comfy_table::{Cell, CellAlignment, Color};
            detail.add_row(vec![
                Cell::new(ui::fmt_size(m.size))
                    .fg(Color::Yellow)
                    .set_alignment(CellAlignment::Right),
                Cell::new(&m.rule_name).fg(Color::Cyan),
                Cell::new(ui::short_path(&m.path)).fg(Color::White),
            ]);
        }
        eprintln!("{detail}");
        if matches.len() > 50 {
            eprintln!(
                "  {} ({} more not shown)",
                ui::dim("..."),
                matches.len() - 50,
            );
        }
    }

    let cleanable: usize = matches.iter().filter(|m| m.clean_command.is_some()).count();
    if cleanable > 0 {
        eprintln!();
        eprintln!(
            "  {}",
            ui::dim(&format!(
                "{cleanable} entries have cleanup commands — run with --exec to clean interactively"
            )),
        );
    }
}

// ---------------------------------------------------------------------------
// Interactive execution
// ---------------------------------------------------------------------------

fn run_interactive(matches: &[SweepMatch]) {
    if matches.is_empty() {
        ui::success("No reclaimable space found.");
        return;
    }

    let mut run_all = false;
    let stdin = io::stdin();
    let mut stdout = io::stderr();

    for m in matches {
        eprintln!();
        eprintln!(
            "  {} {}  {}",
            ui::bold(&ui::short_path(&m.path)),
            ui::dim(&format!("({})", ui::fmt_size(m.size))),
            ui::highlight(&m.rule_name),
        );

        let cmd = match &m.clean_command {
            Some(c) => c,
            None => {
                eprintln!("  {}", ui::dim("(no cleanup command — remove manually)"));
                continue;
            }
        };

        eprintln!("  {} {}", ui::dim("run:"), cmd);

        if !run_all {
            let _ = write!(stdout, "  execute? [y/N/a/q] ");
            let _ = stdout.flush();

            let mut input = String::new();
            if stdin.lock().read_line(&mut input).is_err() {
                break;
            }

            match input.trim().to_lowercase().as_str() {
                "a" => run_all = true,
                "q" => {
                    eprintln!("  {}", ui::dim("quit"));
                    break;
                }
                "y" => {}
                _ => {
                    eprintln!("  {}", ui::dim("skipped"));
                    continue;
                }
            }
        }

        // Execute
        let result = Command::new("sh").arg("-c").arg(cmd).status();

        match result {
            Ok(status) if status.success() => {
                ui::success(&format!("done ({})", ui::fmt_size(m.size)));
            }
            Ok(status) => {
                eprintln!(
                    "  {} exited with {}",
                    ui::dim("command"),
                    status.code().unwrap_or(-1),
                );
            }
            Err(e) => {
                eprintln!("  {} {}", ui::dim("error:"), e);
            }
        }
    }
}
