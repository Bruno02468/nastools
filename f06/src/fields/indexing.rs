//! This submodule implements several indexing types used to acces values in an
//! output block.

use const_format::concatcp;
use serde::{Deserialize, Serialize};

use crate::geometry::{Axis, Dof};

/// All field indexing types must implement this trait.
pub trait IndexType: Copy + Ord + Eq {
  /// The name of this type of index, all caps.
  const INDEX_NAME: &'static str;
}

/// This struct allows one to combine two indexing types into one, like a
/// "combined key" of sorts.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct PairIndex<A: IndexType, B: IndexType> {
  /// The primary, leading index.
  pub primary: A,
  /// The secondary index.
  pub secondary: B
} 

impl<A: IndexType, B: IndexType> IndexType for PairIndex<A, B> {
  const INDEX_NAME: &'static str = A::INDEX_NAME;
}

impl IndexType for Axis {
  const INDEX_NAME: &'static str = "AXIS";
}

impl IndexType for Dof {
  const INDEX_NAME: &'static str = "DOF";
}

/// The possible origins for a force.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub enum ForceOrigin {
  /// The force was applied by a load.
  Load,
  /// The force was applied by another element.
  Element {
    /// The element ID.
    eid: usize
  },
  /// The force was applied by a single-point constraint.
  SinglePointConstraint,
  /// The force was applied by a multi-point constraint.
  MultiPointConstraint
}

impl IndexType for ForceOrigin {
  const INDEX_NAME: &'static str = "FORCE ORIGIN";
}

/// A grid point, referenced by its ID.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct GridPointRef {
  /// The ID of the grid point.
  pub gid: usize
}

impl IndexType for GridPointRef {
  const INDEX_NAME: &'static str = "GRID POINT ID";
}

/// An element, referenced by its ID.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct ElementRef {
  /// The ID of the element.
  pub eid: usize
}

impl IndexType for ElementRef {
  const INDEX_NAME: &'static str = "ELEMENT ID";
}

/// A coordinate system, referenced by its ID.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct CsysRef {
  /// The ID of the coordinate system.
  pub cid: usize
}

impl IndexType for CsysRef {
  const INDEX_NAME: &'static str = "CSYS ID";
}
