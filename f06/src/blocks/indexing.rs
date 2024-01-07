//! This submodule implements several indexing types used to acces values in an
//! output block.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::elements::ElementType;
use crate::geometry::{Axis, Dof};

/// Generates the NasIndex struct that encapsulates all indexing types.
macro_rules! gen_nasindex {
  (
    $($tn:ident,)*
  ) => {
    /// This enum encapsulates all index types, taken generally.
    #[derive(
      Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq,
      PartialOrd, Ord
    )]
    #[allow(missing_docs)] // I refuse.
    pub enum NasIndex {
      $($tn($tn),)*
    }

    $(
      impl From<$tn> for NasIndex {
        fn from(value: $tn) -> Self {
          return Self::$tn(value);
        }
      }
    )*

    impl Display for NasIndex {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
          $(Self::$tn(x) => x.fmt(f),)*
        };
      }
    }
  };
}

gen_nasindex!(
  Axis,
  Dof,
  ForceOrigin,
  GridPointRef,
  ElementRef,
  CsysRef,
  GridPointForceOrigin,
);

/// All field indexing types must implement this trait.
pub trait IndexType: Copy + Ord + Eq + Into<NasIndex> + Display {
  /// The name of this type of index, all caps.
  const INDEX_NAME: &'static str;

  /// Returns a more complex name for the index. Useful if the name is beyond
  /// the reach of const generics.
  fn dyn_name(&self) -> String {
    return Self::INDEX_NAME.to_owned();
  }
}

impl IndexType for Axis {
  const INDEX_NAME: &'static str = "AXIS";
}

impl IndexType for Dof {
  const INDEX_NAME: &'static str = "DOF";
}

/// The possible origins for a force.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
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

impl Display for ForceOrigin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      Self::Load => write!(f, "APPLIED LOAD"),
      Self::Element { elem } => write!(f, "{}", elem),
      Self::SinglePointConstraint => write!(f, "SINGLE-POINT CONSTRAINT"),
      Self::MultiPointConstraint => write!(f, "MULTI-POINT CONSTRAINT"),
    };
  }
}

impl IndexType for ForceOrigin {
  const INDEX_NAME: &'static str = "FORCE ORIGIN";
}

/// A grid point, referenced by its ID.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub struct GridPointRef {
  /// The ID of the grid point.
  pub gid: usize
}

impl Display for GridPointRef {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "GRID {}", self.gid);
  }
}

impl IndexType for GridPointRef {
  const INDEX_NAME: &'static str = "GRID POINT ID";
}

/// An element, referenced by its ID.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub struct ElementRef {
  /// The ID of the element.
  pub eid: usize,
  /// The type of element, if known.
  pub etype: Option<ElementType>
}

impl Display for ElementRef {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self.etype {
      Some(et) => write!(f, "ELEMENT {} ({})", self.eid, et.name()),
      None => write!(f, "ELEMENT {}", self.eid),
    };
  }
}

impl IndexType for ElementRef {
  const INDEX_NAME: &'static str = "ELEMENT ID";
}

/// A coordinate system, referenced by its ID.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub struct CsysRef {
  /// The ID of the coordinate system.
  pub cid: usize
}

impl Display for CsysRef {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "COORD SYS {}", self.cid);
  }
}

impl IndexType for CsysRef {
  const INDEX_NAME: &'static str = "COORD SYS ID";
}

/// A combination of a grid point reference and a force origin.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub struct GridPointForceOrigin {
  /// A reference to the grid point.
  pub grid_point: GridPointRef,
  /// The origin of the force.
  pub force_origin: ForceOrigin
}

impl Display for GridPointForceOrigin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{} FORCE AT {}", self.force_origin, self.grid_point);
  }
}

impl IndexType for GridPointForceOrigin {
  const INDEX_NAME: &'static str = "GRID POINT FORCE ORIGIN";
}
