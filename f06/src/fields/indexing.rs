//! This submodule implements several indexing types used to acces values in an
//! output block.

use serde::{Deserialize, Serialize};

use crate::fields::IndexType;
use crate::geometry::{Axis, Dof};

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
  gid: usize
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
  eid: usize
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
  cid: usize
}

impl IndexType for CsysRef {
  const INDEX_NAME: &'static str = "CSYS ID";
}
