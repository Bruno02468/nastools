//! This module implements the general structure of an F06 file as we interpret
//! it, and its submodules are responsible for specific parsing subroutines.

use std::collections::BTreeSet;

use serde::{Serialize, Deserialize};

use crate::blocks::*;
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

  /// Locates blocks that can be merged and merges them. Returns the number of
  /// done merges. Only does clean merges (no row conflicts).
  pub fn merge_blocks(&mut self) -> usize {
    let mut new_blocks: Vec<FinalBlock> = Vec::new();
    let mut num_merges = 0;
    while let Some(primary) = self.blocks.pop() {
      // look for merge candidates
      let sio: Option<usize> = self.blocks.iter()
        .enumerate()
        .find(|(_, s)| {
          primary.can_merge(s).is_ok() && primary.row_conflicts(s).is_empty()
        }).map(|t| t.0);
      if let Some(si) = sio {
        // at least one to merge
        let secondary = self.blocks.remove(si);
        let merged = match primary.try_merge(secondary) {
          Ok(MergeResult::Success { merged }) => merged,
          _ => panic!("pre-merge check failed!")
        };
        num_merges += 1;
        // put it back since it could have other potential merges
        self.blocks.push(merged);
      } else {
        // unmergeable, put it in the new ones
        new_blocks.push(primary);
      }
    }
    return num_merges;
  }
}
