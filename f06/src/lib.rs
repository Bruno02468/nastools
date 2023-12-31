//! his library implements types and functions to parse and manipulate the
//! data within formatted text output files from Nastran-like FEA solvers.
//! 
//! It was created with the main intent being the development of a tool to
//! convert output from the MYSTRAN solver.to a CSV for use in automated
//! verification of the solver's correctness.
//! 
//! However, the code is modular -- one can easily expand the library to
//! support parsing different "flavours" of text output, different solvers,
//! more elements/formulations, etc.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return)]
#![allow(dead_code)] // temporary

pub mod flavour;
pub mod util;

#[cfg(test)]
mod tests;
