//! This module implements the general structure of an F06 file as we interpret
//! it, and its submodules are responsible for specific parsing subroutines.

use std::collections::BTreeSet;

use serde::{Serialize, Deserialize};

use crate::blocks::FinalBlock;
use crate::flavour::Flavour;

/// This is the output of an F06 parser.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct F06File {
  /// The flavour of file.
  pub flavour: Flavour,
  /// The detected blocks.
  pub blocks: Vec<FinalBlock>,
  /// The line numbers for warning messages.
  pub warnings: BTreeSet<usize>,
  /// The line numbers for fatal error messages.
  pub fatal_errors: BTreeSet<usize>
}

impl Default for F06File {
  fn default() -> Self {
    return Self::new();
  }
}

impl F06File {
  /// Instantiates a new F06 file struct with nothing inside.
  pub fn new() -> Self {
    return Self {
      flavour: Flavour::default(),
      blocks: Vec::new(),
      warnings: BTreeSet::new(),
      fatal_errors: BTreeSet::new()
    };
  }
}
