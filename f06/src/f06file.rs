//! This module implements the general structure of an F06 file as we interpret
//! it, and its submodules are responsible for specific parsing subroutines.

pub mod diff;

use std::collections::{BTreeSet, BTreeMap};

use log::warn;
use serde::{Serialize, Deserialize};

use crate::prelude::*;
use crate::util::*;

/// This is the output of an F06 parser.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct F06File {
  /// The flavour of file.
  pub flavour: Flavour,
  /// The detected blocks.
  pub blocks: Vec<FinalBlock>,
  /// The line numbers for warning messages.
  pub warnings: BTreeMap<usize, String>,
  /// The line numbers for fatal error messages.
  pub fatal_errors: BTreeMap<usize, String>,
  /// Lines with potential, unknown headers, and their line ranges.
  pub potential_headers: BTreeSet<PotentialHeader>
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
      warnings: BTreeMap::new(),
      fatal_errors: BTreeMap::new(),
      potential_headers: BTreeSet::new()
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
    std::mem::swap(&mut new_blocks, &mut self.blocks);
    return num_merges;
  }

  /// Merges the potential headers. Returns the number of merges.
  pub fn merge_potential_headers(&mut self) -> usize {
    let mut new_phs: BTreeSet<PotentialHeader> = BTreeSet::new();
    let mut num_merges: usize = 0;
    while !self.potential_headers.is_empty() {
      // take one
      let first = self.potential_headers.pop_first().unwrap();
      // take another
      if let Some(second) = self.potential_headers.pop_first() {
        // is the next one compatible?
        match first.try_merge(second) {
          Ok(merged) => {
            // merged, put it back, continue.
            self.potential_headers.insert(merged);
            num_merges += 1;
          },
          Err((first, second)) => {
            // couldn't merge. put the first one in the new set, put the second
            // one back, and try again.
            new_phs.insert(first);
            self.potential_headers.insert(second);
          },
        }
      } else {
        // there is no second. put the final one in the new set, and we're done
        new_phs.insert(first);
      }
    }
    std::mem::swap(&mut new_phs, &mut self.potential_headers);
    return num_merges;
  }

  /// Sorts the rows and columns of all blocks.
  pub fn sort_all_blocks(&mut self) {
    for block in self.blocks.iter_mut() {
      block.sort_columns();
      block.sort_rows();
    }
  }

  /// Returns all the subcases.
  pub fn subcases(&self) -> impl Iterator<Item = usize> {
    return self.blocks.iter()
      .map(|b| b.subcase)
      .collect::<BTreeSet<usize>>()
      .into_iter();
  }

  /// Returns all the block types.
  pub fn block_types(&self) -> impl Iterator<Item = BlockType> {
    return self.blocks.iter()
      .map(|b| b.block_type)
      .collect::<BTreeSet<BlockType>>()
      .into_iter();
  }

  /// Searches blocks filtering by subcase and/or type.
  pub fn block_search(
    &self,
    type_filter: Option<BlockType>,
    subcase_filter: Option<usize>
  ) -> impl Iterator<Item = &'_ FinalBlock> {
    return self.blocks.iter()
      .filter(|b| type_filter.is_some_and(|bt| b.block_type == bt))
      .filter(|b| subcase_filter.is_some_and(|sc| b.subcase == sc))
      .collect::<Vec<_>>()
      .into_iter();
  }

  /// An iterator over all blocks that are "unique" in that they are the only
  /// block of their type in their subcase. These are the blocks that should
  /// be used for comparisons.
  pub fn unique_blocks(&self) -> impl Iterator<Item = &FinalBlock> {
    let mut blocks: Vec<&FinalBlock> = Vec::new();
    for subcase in self.subcases() {
      for block_type in self.block_types() {
        let mut found = self.block_search(Some(block_type), Some(subcase))
          .collect::<Vec<_>>();
        match found.len() {
          1 => {
            blocks.push(found.pop().unwrap());
          },
          x if x > 1 => {
            warn!(
              "Subcase {} has {} \"{}\" blocks! This shouldn't happen...",
              subcase,
              found.len(),
              block_type
            );
          },
          _ => {}
        };
      }
    }
    return blocks.into_iter();
  }
}
