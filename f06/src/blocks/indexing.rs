//! This submodule implements several indexing types used to acces values in an
//! output block.

use std::fmt::Display;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::elements::ElementType;
use crate::geometry::{Axis, Dof};

/// Generates a NasIndex type from pure enum fields. Saves some time.
macro_rules! from_enum {
  (
    $desc:literal,
    $tname:ident,
    $tstr:literal,
    [
      $(
        ($varname:ident, $varstr:literal),
      )+
    ]
  ) => {
    #[derive(
      Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq,
      Eq, derive_more::From
    )]
    #[doc = $desc]
    #[allow(missing_docs)]
    pub enum $tname {
      $($varname,)+
    }

    impl $tname {
      /// Returns a short, uppercase name for this index variant.
      pub const fn name(&self) -> &'static str {
        return match self {
          $(Self::$varname => $varstr,)+
        };
      }

      /// Returns all the variants of this index, in canonical order.
      pub const fn all() -> &'static [Self] {
        return &[$(Self::$varname,)+];
      }

      /// Returns a map with this index in canonical order for ease of use when
      /// booting up a decoder.
      pub fn canonical_cols() -> BTreeMap<Self, usize> {
        return Self::all()
          .iter()
          .copied()
          .enumerate()
          .map(|(a, b)| (b, a))
          .collect();
      }
    }

    impl Display for $tname {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.name());
      }
    }

    impl IndexType for $tname {
      const INDEX_NAME: &'static str = $tstr;
    }
  };
}

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
  PointInElement,
  ElementSidedPoint,
  QuadStressField,
  QuadStrainField,
  QuadForcesField,
);

/// All field indexing types must implement this trait.
pub trait IndexType: Copy + Ord + Eq + Into<NasIndex> + Display {
  /// The name of this type of index, all caps.
  const INDEX_NAME: &'static str;
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

/// A point within an element.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq
)]
pub enum ElementPoint {
  /// The element's center.
  Centroid,
  /// A corner point.
  Corner(GridPointRef),
  /// A midpoint.
  Midpoint(GridPointRef)
}

impl Display for ElementPoint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      ElementPoint::Centroid => write!(f, "CENTROID"),
      ElementPoint::Corner(GridPointRef { gid }) => {
        write!(f, "CORNER AT GRID {}", gid)
      },
      ElementPoint::Midpoint(GridPointRef { gid }) => {
        write!(f, "MIDPOINT AT GRID {}", gid)
      }
    };
  }
}

/// An element side.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub enum ElementSide {
  /// The bottom (Z1) side of the element.
  Bottom,
  /// The top (Z2) side of the element.
  Top
}

impl Display for ElementSide {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{} SIDE", match self {
      Self::Bottom => "BOTTOM",
      Self::Top => "TOP",
    });
  }
}

impl ElementSide {
  /// Returns the opposite side.
  pub const fn opposite(&self) -> Self {
    return match self {
      Self::Bottom => Self::Top,
      Self::Top => Self::Bottom,
    };
  }
}

/// An element and a point within it.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub struct PointInElement {
  /// A reference to the element.
  pub element: ElementRef,
  /// The point within the element.
  pub point: ElementPoint
}

impl Display for PointInElement {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}, {}", self.element, self.point);
  }
}

impl IndexType for PointInElement {
  const INDEX_NAME: &'static str = "POINT IN ELEMENT";
}

/// An element and a point within it, plus a side.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
pub struct ElementSidedPoint {
  /// A reference to the element.
  pub element: ElementRef,
  /// The point within the element.
  pub point: ElementPoint,
  /// The side.
  pub side: ElementSide
}

impl Display for ElementSidedPoint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}, {}, {}", self.element, self.point, self.side);
  }
}

impl IndexType for ElementSidedPoint {
  const INDEX_NAME: &'static str = "ELEMENT, POINT AND SIDE";
}

impl ElementSidedPoint {
  /// Flips the side of this element point.
  pub fn flip_side(&mut self) {
    self.side = self.side.opposite();
  }
}

from_enum!(
  "The columns for the stress table for a quadrilateral element.",
  QuadStressField,
  "QUAD STRESS FIELD",
  [
    (FibreDistance, "FIBRE DISTANCE"),
    (NormalX, "NORMAL-X"),
    (NormalY, "NORMAL-Y"),
    (ShearXY, "SHEAR-XY"),
    (Angle, "ANGLE"),
    (Major, "MAJOR"),
    (Minor, "MINOR"),
    (VonMises, "VON MISES"),
  ]
);

/// The columns for the strain table for a quadrilateral element.
#[derive(
  Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq,
  derive_more::From
)]
#[allow(missing_docs)] // nah
pub struct QuadStrainField(QuadStressField);

impl Display for QuadStrainField {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return self.0.fmt(f);
  }
}

impl IndexType for QuadStrainField {
  const INDEX_NAME: &'static str = "QUAD STRAIN FIELD";
}

from_enum!(
  "The columns for the engineering forces table for a quadrilateral element.",
  QuadForcesField,
  "QUAD FORCE FIELD",
  [
    (NormalX, "Nx"),
    (NormalY, "Ny"),
    (NormalXY, "Nxy"),
    (MomentX, "Mx"),
    (MomentY, "My"),
    (MomentXY, "Mxy"),
    (TransverseShearX, "Qx"),
    (TransverseShearY, "Qy"),
  ]
);
