//! This submodule implements the general layout of CSV files.

#![allow(clippy::needless_return)] // i'll never forgive rust for this

use std::fmt::Display;

use clap::ValueEnum;
use f06::prelude::*;
use serde::{Serialize, Deserialize};

/// Number of fields in a fixed-form CSV record.
pub const NAS_CSV_COLS: usize = 11;

/// Type that holds the headers for a row.
pub type RowHeader = [&'static str; NAS_CSV_COLS-1];

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
  EngForces,
  /// The 5-block: grid point force balance.
  GridPointForces
}

impl CsvBlockId {
  /// Returns a constant name for this block ID.
  pub const fn name(&self) -> &'static str {
    return match self {
      Self::SolInfo => "SolutionInfo",
      Self::Displacements => "Displacements",
      Self::Stresses => "Stresses",
      Self::Strains => "Strains",
      Self::EngForces => "EngForces",
      Self::GridPointForces => "GridPointForces"
    };
  }

  /// Returns the name with "block ID" appended. Needed because headers gotta
  /// be &'static str.
  pub const fn name_with_id(&self) -> &'static str {
    return match self {
      Self::SolInfo => "SolutionInfo block ID",
      Self::Displacements => "Displacements block ID",
      Self::Stresses => "Stresses block ID",
      Self::Strains => "Strains block ID",
      Self::EngForces => "EngForces block ID",
      Self::GridPointForces => "GridPointForces block ID"
    };
  }
}

impl Display for CsvBlockId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", self.name());
  }
}

impl From<CsvBlockId> for usize {
  fn from(value: CsvBlockId) -> Self {
    return match value {
      CsvBlockId::SolInfo => 0,
      CsvBlockId::Displacements => 1,
      CsvBlockId::Stresses => 2,
      CsvBlockId::Strains => 3,
      CsvBlockId::EngForces => 4,
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
      4 => CsvBlockId::EngForces,
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
#[derive(Clone, Debug, Serialize)]
pub struct CsvRecord {
  /// The CSV block type.
  pub block_id: CsvBlockId,
  /// Block type that originated this record. If none, it's the 0-block.
  pub block_type: Option<BlockType>,
  /// If this record relates to a grid point, its ID.
  pub gid: Option<usize>,
  /// If this record relates to an element, the element ID.
  pub eid: Option<usize>,
  /// If this record relates to an element, its type.
  pub etype: Option<ElementType>,
  /// If this record relates to a subcase, its ID.
  pub subcase: Option<usize>,
  /// The remaining ten fields.
  pub fields: [CsvField; NAS_CSV_COLS-1],
  /// The headers for the ten fields.
  pub headers: &'static RowHeader
}

impl CsvRecord {
  /// Returns this as eleven strings.
  pub fn to_fields(self) -> impl Iterator<Item = CsvField> {
    return [CsvField::from(self.block_id)].into_iter().chain(self.fields);
  }

  /// Returns this block's headers as eleven strings.
  pub fn header_as_iter(&self) -> impl Iterator<Item = &str> {
    return [self.block_id.name_with_id()].into_iter().chain(
      self.headers.iter().copied()
    );
  }
}
