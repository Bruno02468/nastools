//! This module implements data structures to specify ways to extract data
//! subsets from F06 files.

use std::error::Error;
use std::fmt::Display;
use std::mem::discriminant;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// This specifies a value or sets thereof.
#[derive(
  Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub enum Specifier<A> {
  /// Use all in the file.
  All,
  /// Use a list.
  List(Vec<A>),
  /// Use an exclusion list.
  AllExcept(Vec<A>)
}

impl<A: PartialEq> Specifier<A> {
  /// Use this as a filter for an iterator.
  fn filter_fn(&self, item: &A) -> bool {
    return match self {
      Self::All => true,
      Self::List(l) => l.contains(item),
      Self::AllExcept(l) => !l.contains(item),
    };
  }
}

/// This is a "full index", it refers to a single datum in an F06 file.
#[derive(
  Debug, Copy, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct DatumIndex {
  /// The block reference.
  pub block_ref: BlockRef,
  /// The row.
  pub row: NasIndex,
  /// The column.
  pub col: NasIndex
}

impl DatumIndex {
  /// Attempts to get the value at this index from a data block.
  pub fn get_from(
    &self,
    file: &F06File
  ) -> Result<F06Number, ExtractionError> {
    let block = file.block_search(
      Some(self.block_ref.block_type),
      Some(self.block_ref.subcase),
      true
    ).nth(0).ok_or(ExtractionError::NoSuchBlock(self.block_ref))?;
    let ri_ex = block.row_indexes.keys().nth(0)
      .ok_or(ExtractionError::BlockIsEmpty)?;
    let ci_ex = block.col_indexes.keys().nth(0)
      .ok_or(ExtractionError::BlockIsEmpty)?;
    if discriminant(&self.row) != discriminant(ri_ex) {
      return Err(ExtractionError::RowTypeMismatch {
        tried: self.row,
        against: *ri_ex
      });
    }
    if discriminant(&self.col) != discriminant(ci_ex) {
      return Err(ExtractionError::ColumnTypeMismatch {
        tried: self.col,
        against: *ci_ex
      });
    }
    if !block.row_indexes.contains_key(&self.row) {
      return Err(ExtractionError::MissingRow(self.row));
    }
    if !block.col_indexes.contains_key(&self.col) {
      return Err(ExtractionError::MissingColumn(self.col));
    }
    return Ok(block.get(self.row, self.col).expect("row & col check failed!"));
  }
}

/// This is the kind of error that can be returned when extracting a datum.
#[derive(
  Debug, Copy, Clone, Serialize, Deserialize
)]
pub enum ExtractionError {
  /// The F06 file has no block matching a subcase and type.
  NoSuchBlock(BlockRef),
  /// The block was found but there was an index type mismatch for the rows.
  RowTypeMismatch {
    /// A row index we tried to use.
    tried: NasIndex,
    /// An example of row index from the block.
    against: NasIndex
  },
  /// The block was found but there was an index type mismatch for the columns.
  ColumnTypeMismatch {
    /// A column index we tried to use.
    tried: NasIndex,
    /// An example of column index from the block.
    against: NasIndex
  },
  /// A row index is the correct type but is not part of the matrix.
  MissingRow(NasIndex),
  /// A column index is the correct type but is not part of the matrix.
  MissingColumn(NasIndex),
  /// The block has no data.
  BlockIsEmpty
}

impl Display for ExtractionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      Self::NoSuchBlock(bref) => write!(
        f,
        "no such block ({}, subcase {})",
        bref.block_type.short_name(),
        bref.subcase
      ),
      Self::RowTypeMismatch { tried, against } => write!(
        f,
        "wrong row type (tried a {}, block uses {})",
        tried.type_name(),
        against.type_name()
      ),
      Self::ColumnTypeMismatch { tried, against } => write!(
        f,
        "wrong column type (tried a {}, block uses {})",
        tried.type_name(),
        against.type_name()
      ),
      Self::MissingRow(ri) => write!(f, "no such row ({})", ri),
      Self::MissingColumn(ci) => write!(f, "no such column ({})", ci),
      Self::BlockIsEmpty => write!(f, "block is empty")
    };
  }
}

impl Error for ExtractionError {}

/// This structure represents a way to extract a subset of the data from an F06
/// so one can apply comparison criteria to it.
#[derive(
  Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct Extraction {
  /// Subcases to get data from.
  subcases: Specifier<usize>,
  /// Block types to get data from.
  blocks: Specifier<BlockType>,
  /// Grid point filter (filters out grid points if present).
  grid_points: Specifier<GridPointRef>,
  /// Element filter (filters out element IDs if present).
  elements: Specifier<ElementRef>,
  /// Row filter (for when you want very specific data).
  rows: Specifier<NasIndex>,
  /// Column filter (for when you want very specific data).
  cols: Specifier<NasIndex>
}

impl Extraction {
  /// Produces an iterator over the indices resulting from applying an
  /// extraction to a file. This assumes the file has already had its blocks
  /// sorted and merged.
  pub fn lookup(
    &self,
    file: &F06File
  ) -> impl Iterator<Item = F06Number> + '_ {
    todo!()
  }
}
