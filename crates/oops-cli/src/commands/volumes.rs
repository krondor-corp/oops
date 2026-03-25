//! Volumes command -- mounted filesystem details

use clap::Args;

use oops_core::list_volumes;

use crate::op::{Ctx, NoOutput, Op};
use crate::ui;

/// Show mounted volumes and disk partitions
#[derive(Args, Debug, Clone)]
pub struct Volumes;

#[derive(Debug, thiserror::Error)]
pub enum VolumesError {
    #[error(transparent)]
    Core(#[from] oops_core::Error),
}

impl Op for Volumes {
    type Error = VolumesError;
    type Output = NoOutput;

    fn run(&self, _ctx: &Ctx) -> Result<Self::Output, Self::Error> {
        let volumes = list_volumes()?;
        ui::render_volumes(&volumes);
        Ok(NoOutput)
    }
}
