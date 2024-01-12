//! This module implements a list of known data blocks and information related
//! to them, such as names for detection and decoder instantiation subroutines.

use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::blocks::{BlockDecoder, OpaqueDecoder};
use crate::blocks::decoders::*;
use crate::flavour::Flavour;

/// Generates the BlockType enum and calls the init functions for them.
macro_rules! gen_block_types {
  (
    $(
      {
        $desc:literal,
        $bname:ident,
        $dec:ty,
        $spaceds:expr
      },
    )*
  ) => {
    /// This contains all the known data blocks.
    #[derive(
      Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd,
      Ord
    )]
    #[non_exhaustive]
    pub enum BlockType {
      $(
        #[doc = $desc]
        $bname,
      )*
    }

    impl BlockType {
      /// Returns all known block types.
      pub const fn all() -> &'static [Self] {
        return &[ $(Self::$bname,)* ];
      }

      /// Instantiates the decoder for this data block type.
      pub fn init_decoder(&self, flavour: Flavour) -> Box<dyn OpaqueDecoder> {
        return match self {
          $(
            Self::$bname => Box::from(<$dec as BlockDecoder>::new(flavour)),
          )*
        };
      }

      /// Returns the name of the block.
      pub const fn desc(&self) -> &'static str {
        return match self {
          $(
            Self::$bname => $desc,
          )*
        };
      }

      /// Returns the known upper-case, "spaced" forms that signal the
      /// beginning of this block.
      pub fn headers(&self) -> &'static [&'static str] {
        return match self {
          $(
            Self::$bname => &$spaceds,
          )*
        };
      }
    }
  }
}

gen_block_types!(
  {
    "Grid point displacements",
    Displacements,
    DisplacementsDecoder,
    ["DISPLACEMENTS", "DISPLACEMENT VECTOR"]
  },
  {
    "Grid point force balance",
    GridPointForceBalance,
    GridPointForceBalanceDecoder,
    ["GRID POINT FORCE BALANCE"]
  },
  {
    "Forces of single-point constraint",
    SpcForces,
    SpcForcesDecoder,
    ["SPC FORCES", "FORCES OF SINGLE-POINT CONSTRAINT"]
  },
  {
    "Applied forces",
    AppliedForces,
    AppliedForcesDecoder,
    ["APPLIED FORCES", "LOAD VECTOR"]
  },
  {
    "Stresses in quadrilateral elements",
    QuadStresses,
    QuadStressesDecoder,
    [
      "STRESSES IN QUADRILATERAL ELEMENTS",
      concat!(
        "ELEMENT STRESSES IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE QUAD4"
      )
    ]
  },
  {
    "Strains in quadrilateral elements",
    QuadStrains,
    QuadStrainsDecoder,
    [
      "STRAINS IN QUADRILATERAL ELEMENTS",
      concat!(
        "ELEMENT STRAINS IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE QUAD4"
      )
    ]
  },
  {
    "Engineering forces in quadrilateral elements",
    QuadForces,
    QuadForcesDecoder,
    [
      "FORCES IN QUADRILATERAL ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE QUAD4"
    ]
  },
  {
    "Engineering forces in triangular elements",
    TriForces,
    TriForcesDecoder,
    [
      "FORCES IN TRIANGULAR ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE TRIA3"
    ]
  },
  {
    "Engineering forces in rod elements",
    RodForces,
    RodForcesDecoder,
    [
      "FORCES IN ROD ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE ROD"
    ]
  },
  {
    "Engineering forces in bar elements",
    BarForces,
    BarForcesDecoder,
    [
      "FORCES IN BAR ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE BAR"
    ]
  },
);

impl Display for BlockType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", self.desc());
  }
}
