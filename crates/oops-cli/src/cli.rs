use std::path::PathBuf;

use clap::Parser;

use crate::commands;

#[derive(Parser, Debug)]
#[command(
    name = "oops",
    about = "Fast disk usage diagnostics for Unix systems",
    version,
    after_help = "Use 'oops <command> --help' for more information about a command."
)]
pub struct Cli {
    /// Show verbose output
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    /// Plain output: no colors, no decorations (for scripting)
    #[arg(long = "plain", global = true)]
    pub plain: bool,

    #[command(subcommand)]
    pub command: Option<Command>,

    /// Target path to analyze (default: current directory)
    #[arg(global = false)]
    pub path: Option<PathBuf>,

    /// Maximum depth levels to display
    #[arg(short, long, default_value = "4")]
    pub depth: usize,

    /// Max entries shown per directory level
    #[arg(short = 'n', long, default_value = "5")]
    pub show: usize,

    /// Max children to recurse into per level
    #[arg(long, default_value = "3")]
    pub expand: usize,

    /// Terminal width override (default: auto-detect)
    #[arg(long)]
    pub width: Option<usize>,
}

crate::command_enum! {
    #[command(visible_alias = "f")]
    (Free, commands::Free),

    (Update, commands::Update),
}
