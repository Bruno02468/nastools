//! This library implements types and functions to parse and manipulate the
//! data within formatted text output files from Nastran-like FEA solvers.
//!
//! It was created with the main intent being the development of a tool to
//! convert output from the MYSTRAN solver.to a CSV for use in automated
//! verification of the solver's correctness.
//!
//! However, the code is modular -- one can easily expand the library to
//! support parsing different "flavours" of text output, different solvers,
//! more elements/formulations, etc.

#![warn(missing_docs)] // almost sure this is default but whatever
#![warn(clippy::missing_docs_in_private_items)] // sue me
#![allow(clippy::needless_return)] // i'll never forgive rust for this
#![allow(dead_code)] // temporary

pub mod blocks;
pub mod elements;
pub mod f06file;
pub mod flavour;
pub mod geometry;
pub mod parser;
pub mod util;

/// Prelude module; includes commonly-used public exports.
pub mod prelude {
  pub use crate::blocks::compare::*;
  pub use crate::blocks::indexing::*;
  pub use crate::blocks::types::*;
  pub use crate::blocks::*;
  pub use crate::elements::*;
  pub use crate::f06file::diff::*;
  pub use crate::f06file::extraction::*;
  pub use crate::f06file::*;
  pub use crate::flavour::*;
  pub use crate::geometry::*;
  pub use crate::parser::*;
}

#[cfg(test)]
mod tests;
