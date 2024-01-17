//! This library implements a highly-configurable CSV format to convert Nastran
//! output to. Primarily meant for use by the `f06csv` tool, i.e. F06 to CSV
//! conversion.

#![allow(clippy::needless_return)]
#[warn(missing_docs)]
#[warn(clippy::missing_docs_in_private_items)]

pub mod from_f06;
pub mod layout;


/// Imports the most relevant exports from the library.
pub mod prelude {
  pub use super::from_f06::*;
  pub use super::layout::*;
}
