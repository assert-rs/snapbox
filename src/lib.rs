mod cargo;
mod cases;
mod color;
mod command;
mod runner;
pub mod schema;
mod spec;

pub use cases::TestCases;

pub(crate) use cargo::cargo_bin;
pub(crate) use color::Palette;
pub(crate) use command::wait_with_input_output;
pub(crate) use runner::{Case, Mode, Runner};
pub(crate) use schema::{Bin, CommandStatus, Env, TryCmd};
pub(crate) use spec::RunnerSpec;
