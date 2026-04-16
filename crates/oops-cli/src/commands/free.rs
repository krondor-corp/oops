//! Free command -- quick one-liner: how much space is left on this disk?

use std::path::PathBuf;

use clap::Args;

use oops_core::list_volumes;

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Show free space on the volume containing the current directory
#[derive(Args, Debug, Clone)]
pub struct Free {
    /// Path to check (default: current directory)
    pub path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum FreeError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
    #[error("could not determine volume for {0}")]
    NoVolume(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Op for Free {
    type Error = FreeError;
    type Output = NoOutput;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let target = self.path.as_ref().unwrap_or(&ctx.path);
        let canonical = std::fs::canonicalize(target)?;

        let volumes = list_volumes()?;
        let volume = volumes
            .iter()
            .filter(|v| canonical.starts_with(&v.mount_point))
            .max_by_key(|v| v.mount_point.as_os_str().len())
            .ok_or_else(|| FreeError::NoVolume(canonical.clone()))?;

        ui::render_free(volume);
        Ok(NoOutput)
    }
}
