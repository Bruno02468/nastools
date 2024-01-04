//! This submodule implements several indexing types used to acces values in an
//! output block.

use serde::{Deserialize, Serialize};

use crate::geometry::{Axis, Dof};

/// All field indexing types must implement this trait.
pub trait IndexType: Copy + Ord + Eq {
  /// The name of this type of index, all caps.
  const INDEX_NAME: &'static str;

  /// Returns a more complex name for the index. Useful if the name is beyond
  /// the reach of const generics.
  fn dyn_name(&self) -> String {
    return Self::INDEX_NAME.to_owned();
  }
}

/// This enum encapsulates all index types, taken generally.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord
)]
#[allow(missing_docs)] // I refuse.
pub enum NasIndex {
  Axis(Axis),
  Dof(Dof),
  ForceOrigin(ForceOrigin),
  GridPointRef(GridPointRef),
  ElementRef(ElementRef),
  CsysRef(CsysRef)
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
    /// A reference to the element.
    elem: ElementRef
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
  // The type of element, if known.
  
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

/// A grid point within a coordinate system.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub struct GridInCsys {
  /// The ID of the grid point.
  pub gid: usize,
  /// The ID of the coordinate system.
  pub cid: usize
}
