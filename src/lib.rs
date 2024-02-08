#[macro_use]
extern crate log;
// #[macro_use]
// extern crate miette;
#[cfg(test)]
#[macro_use]
extern crate insta;
#[macro_use]
pub mod error;
pub mod complete;
pub mod context;
pub(crate) mod env;
pub mod parse;

pub use crate::parse::arg::SpecArg_old;
pub use crate::parse::cmd::SpecCommand_old;
pub use crate::parse::command::Command;
pub use crate::parse::flag::SpecFlag_old;
pub use crate::parse::spec::{Spec_old, CLI};
