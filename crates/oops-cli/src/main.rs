//! oops - Fast disk usage diagnostics for Unix systems

#[macro_use]
mod op;

mod cli;
mod commands;
mod ui;

use std::io::IsTerminal;
use std::path::PathBuf;

use clap::Parser;

use cli::Cli;
use op::{Ctx, Op};

fn main() {
    // Parse CLI first so we can use --verbose to set the tracing level.
    // tracing is initialized below, after we know the verbosity.
    let cli = Cli::parse();

    let default_filter = if cli.verbose { "debug" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_filter)),
        )
        .with_writer(std::io::stderr)
        .init();

    if let Err(e) = run(cli) {
        ui::print_error(e.as_ref());
        std::process::exit(1);
    }
}

fn resolve_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    ui::set_plain(cli.plain);

    if !cli.plain && std::io::stderr().is_terminal() {
        colored::control::set_override(true);
    }

    let explicit_path = cli.path.is_some();
    let ctx = Ctx {
        path: resolve_path(cli.path),
        explicit_path,
    };

    match cli.command {
        None => {
            let overview = commands::Overview { path: None };
            overview.run(&ctx)?;
            Ok(())
        }
        Some(ref command) => {
            let output = command.run(&ctx)?;
            let output_str = output.to_string();
            if !output_str.is_empty() {
                println!("{}", output_str);
            }
            Ok(())
        }
    }
}
