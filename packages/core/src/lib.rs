#![deny(clippy::all)]

mod ast;
mod bundle;
mod external_module;
mod graph;
mod hook_driver;
mod module;
mod statement;
mod types;
mod utils;

pub use bundle::*;
pub use graph::*;
pub use hook_driver::*;
pub use module::*;
pub use statement::*;
pub use types::module::RollDownModule;

pub use swc_common;
