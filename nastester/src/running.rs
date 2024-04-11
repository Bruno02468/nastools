//! This defines subroutines to run decks and do test runs.

use std::error::Error;
use core::fmt::Display;
use std::path::PathBuf;

use f06::prelude::*;
use serde::{Deserialize, Serialize};


/// This is how we run a solver, if at all, to acquire an F06 file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum RunMethod {
  /// The F06 file is directly imported from a directory, containing the .F06
  /// files with the same base name as the decks.
  ImportFromDir(PathBuf),
  /// A solver is run passing the deck as an argument, and the F06 is got from
  /// reading the same
  RunSolver(PathBuf)
}

/// These are the errors that can come up when running a solver to get the F06
/// output.
#[derive(Debug)]
pub(crate) enum RunError {
  /// The F06 was not found at its supposed location.
  MissingF06(PathBuf),
  /// The F06 was found but could not be read.
  UnreadableF06(PathBuf, Box<dyn Error>)
}

impl Display for RunError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      RunError::MissingF06(p) => write!(f, "missing F06 file at {}", p),
      RunError::UnreadableF06(p, e) => write!(
        f,
        "could not read F06 file at {}, reason: {}",
        p,
        e
      ),
    }
  }
}

/// This is a named "F06 acquisition method". A solver, for short.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RunnableSolver {
  /// The kind of solver. Must be supported by the F06 library.
  pub(crate) kind: Solver,
  /// The "nickname" for this solver, so you can tell versions apart.
  pub(crate) nickname: String,
  /// The method through which we actually get an F06.
  pub(crate) method: RunMethod
}
