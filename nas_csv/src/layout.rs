//! This submodule implements the general layout of CSV files.

#![allow(clippy::needless_return)] // i'll never forgive rust for this

use std::fmt::Display;

use clap::ValueEnum;
use f06::prelude::*;
use serde::{Serialize, Deserialize};

/// Number of fields in a fixed-form CSV record.
pub const NAS_CSV_COLS: usize = 11;

/// CSV block IDs based on their content.]
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
  ValueEnum
)]
#[clap(rename_all = "snake_case")]
#[non_exhaustive]
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
  GridPointForces
}

impl From<CsvBlockId> for usize {
  fn from(value: CsvBlockId) -> Self {
    return match value {
      CsvBlockId::SolInfo => 0,
      CsvBlockId::Displacements => 1,
      CsvBlockId::Stresses => 2,
      CsvBlockId::Strains => 3,
      CsvBlockId::Forces => 4,
      CsvBlockId::GridPointForces => 5
    };
  }
}

impl From<CsvBlockId> for CsvField {
  fn from(value: CsvBlockId) -> Self {
    return Self::Natural(value.into());
  }
}

impl TryFrom<usize> for CsvBlockId {
  type Error = ();

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    return Ok(match value {
      0 => CsvBlockId::SolInfo,
      1 => CsvBlockId::Displacements,
      2 => CsvBlockId::Stresses,
      3 => CsvBlockId::Strains,
      4 => CsvBlockId::Forces,
      5 => CsvBlockId::GridPointForces,
      _ => return Err(())
    });
  }
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
  /// An alloc'd string.
  String(String),
  /// An element type.
  ElementType(ElementType)
}

impl From<F06Number> for CsvField {
  fn from(value: F06Number) -> Self {
    return match value {
      F06Number::Real(x) => Self::Real(x),
      F06Number::Integer(i) => Self::Integer(i),
      F06Number::Natural(n) => Self::Natural(n)
    };
  }
}

impl Display for CsvField {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      Self::Blank => write!(f, ""),
      Self::Integer(i) => i.fmt(f),
      Self::Natural(n) => n.fmt(f),
      Self::Real(x) => x.fmt(f),
      Self::String(s) => s.fmt(f),
      Self::ElementType(et) => et.fmt(f)
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
  /// If this record relates to an element, the element ID.
  pub eid: Option<usize>,
  /// If this record relates to an element, its type.
  pub etype: Option<ElementType>,
  /// If this record relates to a grid point, its ID.
  pub gid: Option<usize>,
  /// The remaining ten fields.
  pub fields: [CsvField; NAS_CSV_COLS-1]
}

impl CsvRecord {
  /// Returns this as eleven strings.
  pub fn to_fields(self) -> impl Iterator<Item = CsvField> {
    return [CsvField::from(self.block_id)].into_iter().chain(self.fields);
  }
}
