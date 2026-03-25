//! Op trait -- typed command pattern for CLI operations.

use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

pub struct Ctx {
    pub path: PathBuf,
    pub explicit_path: bool,
}

pub trait Op {
    type Error: Error + Send + Sync + 'static;
    type Output: Display;

    fn run(&self, ctx: &Ctx) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Default)]
pub struct NoOutput;

impl Display for NoOutput {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[macro_export]
macro_rules! command_enum {
    ($($(#[$attr:meta])* ($variant:ident, $type:ty)),* $(,)?) => {
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum Command {
            $(
                $(#[$attr])*
                $variant($type),
            )*
        }

        #[derive(Debug)]
        pub enum OpOutput {
            $($variant(<$type as $crate::op::Op>::Output),)*
        }

        #[derive(Debug, thiserror::Error)]
        pub enum OpError {
            $(
                #[error(transparent)]
                $variant(<$type as $crate::op::Op>::Error),
            )*
        }

        impl $crate::op::Op for Command {
            type Output = OpOutput;
            type Error = OpError;

            fn run(&self, ctx: &$crate::op::Ctx) -> Result<Self::Output, Self::Error> {
                match self {
                    $(
                        Command::$variant(op) => {
                            op.run(ctx)
                                .map(OpOutput::$variant)
                                .map_err(OpError::$variant)
                        },
                    )*
                }
            }
        }

        impl std::fmt::Display for OpOutput {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        OpOutput::$variant(output) => write!(f, "{}", output),
                    )*
                }
            }
        }
    };
}
