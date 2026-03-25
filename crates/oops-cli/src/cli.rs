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
}

crate::command_enum! {
    #[command(visible_alias = "o")]
    (Overview, commands::Overview),

    #[command(visible_alias = "vol")]
    (Volumes, commands::Volumes),

    #[command(visible_alias = "t")]
    (Top, commands::Top),

    (Tree, commands::Tree),

    #[command(visible_alias = "s")]
    (Sweep, commands::Sweep),

    #[command(visible_alias = "d")]
    (Drill, commands::Drill),
}
