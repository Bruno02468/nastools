//! This submodule implements the general layout of CSV files.

#![allow(clippy::needless_return)] // i'll never forgive rust for this

use std::fmt::Display;

use f06::prelude::*;
use serde::{Serialize, Deserialize};

/// Number of fields in a fixed-form CSV record.
pub const NAS_CSV_COLS: usize = 11;

/// CSV block IDs based on their content.]
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord
)]
pub enum CsvBlockId {
  /// The 0-block: general solution info; subcase IDs, solution types, etc.
  SolInfo,
  /// The 1-block: displacements.
  Displacements,
  /// The 2-block: stresses.
  Stresses,
  /// The 3-block: strains.
  Strains,
  /// The 4-block: forces.
  Forces,
  /// The 5-block: grid point force balance.
  GridPointForceBalance
}

/// The kinds of CSV records we can find in our format.
#[derive(
  Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd,
  derive_more::From
)]
pub enum CsvField {
  /// A blank record.
  Blank,
  /// An integer.
  Integer(isize),
  /// A natural number.
  Natural(usize),
  /// A real number.
  Real(f64),
  /// A Nastran index.
  NasIndex(NasIndex)
}

impl Display for CsvField {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      Self::Blank => write!(f, ""),
      Self::Integer(i) => i.fmt(f),
      Self::Natural(n) => n.fmt(f),
      Self::Real(x) => x.fmt(f),
      Self::NasIndex(ix) => ix.fmt(f)
    };
  }
}

/// A non-header line in a CSV file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvRecord {
  /// The CSV block type.
  pub block_id: CsvBlockId,
  /// Block type that originated this record. If none, it's the 0-block.
  pub block_type: Option<BlockType>,
  /// If this record relates to an element
  /// The remaining ten fields.
  pub fields: [CsvField; NAS_CSV_COLS-1]
}
