//! This module implements a list of known data blocks and information related
//! to them, such as names for detection and decoder instantiation subroutines.

use std::fmt::Display;

use serde::{Serialize, Deserialize};
use convert_case::{Case, Casing};

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
          $(Self::$bname => $desc,)*
        };
      }

      /// Returns the known upper-case, "spaced" forms that signal the
      /// beginning of this block.
      pub fn headers(&self) -> &'static [&'static str] {
        return match self {
          $(Self::$bname => &$spaceds,)*
        };
      }

      /// Returns the small name of the variant, CamelCase.
      pub const fn short_name(&self) -> &'static str {
        return match self {
          $(Self::$bname => stringify!($bname),)*
        };
      }

      /// Returns the small, snake case name of the variant.
      pub fn snake_case_name(&self) -> String {
        return match self {
          $(Self::$bname => self.short_name().to_case(Case::Snake),)*
        };
      }
    }
  }
}

gen_block_types!(
  // displacements
  {
    "Grid point displacements",
    Displacements,
    DisplacementsDecoder,
    ["DISPLACEMENTS", "DISPLACEMENT VECTOR"]
  },
  // grid point force balance
  {
    "Grid point force balance",
    GridPointForceBalance,
    GridPointForceBalanceDecoder,
    ["GRID POINT FORCE BALANCE"]
  },
  // spc forces
  {
    "Forces of single-point constraint",
    SpcForces,
    SpcForcesDecoder,
    ["SPC FORCES", "FORCES OF SINGLE-POINT CONSTRAINT"]
  },
  // applied forces
  {
    "Applied forces",
    AppliedForces,
    AppliedForcesDecoder,
    ["APPLIED FORCES", "LOAD VECTOR"]
  },
  // elas1 forces
  {
    "Engineering forces in ELAS1 elements",
    Elas1Forces,
    Elas1ForcesDecoder,
    [
      "FORCES IN SCALAR SPRINGS (CELAS1)",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE ELAS1"
    ]
  },
  // elas1 stresses
  {
    "Stresses in ELAS1 elements",
    Elas1Stresses,
    Elas1StressesDecoder,
    [
      "STRESSES IN SCALAR SPRINGS (CELAS1)",
      concat!(
        "ELEMENT STRESSES IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE ELAS1"
      )
    ]
  },
  // elas1 strains
  {
    "Strains in ELAS1 elements",
    Elas1Strains,
    Elas1StrainsDecoder,
    [
      "STRAINS IN SCALAR SPRINGS (CELAS1)",
      concat!(
        "ELEMENT STRAINS IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE ELAS1"
      )
    ]
  },
  // rod forces
  {
    "Engineering forces in rod elements",
    RodForces,
    RodForcesDecoder,
    [
      "FORCES IN ROD ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE ROD"
    ]
  },
  // rod stresses
  {
    "Stresses in rod elements",
    RodStresses,
    RodStressesDecoder,
    [
      "STRESSES IN ROD ELEMENTS (CROD)",
      concat!(
        "ELEMENT STRESSES IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE ROD"
      )
    ]
  },
  // rod strains
  {
    "Strains in rod elements",
    RodStrains,
    RodStrainsDecoder,
    [
      "STRAINS IN ROD ELEMENTS (CROD)",
      concat!(
        "ELEMENT STRAINS IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE ROD"
      )
    ]
  },
  // bar forces
  {
    "Engineering forces in bar elements",
    BarForces,
    BarForcesDecoder,
    [
      "FORCES IN BAR ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE BAR"
    ]
  },
  // bar stresses
  {
    "Stresses in bar elements",
    BarStresses,
    BarStressesDecoder,
    [
      "STRESSES IN BAR ELEMENTS (CBAR)",
      concat!(
        "ELEMENT STRESSES IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE BAR"
      )
    ]
  },
  // bar strains
  {
    "Strains in bar elements",
    BarStrains,
    BarStrainsDecoder,
    [
      "STRAINS IN BAR ELEMENTS (CBAR)",
      concat!(
        "ELEMENT STRAINS IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE BAR"
      )
    ]
  },
  // tria forces
  {
    "Engineering forces in triangular elements",
    TriaForces,
    TriaForcesDecoder,
    [
      "FORCES IN TRIANGULAR ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE TRIA3"
    ]
  },
  // tria stresses
  {
    "Stresses in triangular elements",
    TriaStresses,
    TriaStressesDecoder,
    [
      "STRESSES IN TRIANGULAR ELEMENTS (TRIA3)",
      concat!(
        "ELEMENT STRESSES IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE TRIA3"
      )
    ]
  },
  // tria strains
  {
    "Strains in triangular elements",
    TriaStrains,
    TriaStrainsDecoder,
    [
      "STRAINS IN TRIANGULAR ELEMENTS (TRIA3)",
      concat!(
        "ELEMENT STRAINS IN LOCAL ELEMENT COORDINATE SYSTEM ",
        "FOR ELEMENT TYPE TRIA3"
      )
    ]
  },
  // quad stresses
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
  // quad strains
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
  // quad forces
  {
    "Engineering forces in quadrilateral elements",
    QuadForces,
    QuadForcesDecoder,
    [
      "FORCES IN QUADRILATERAL ELEMENTS",
      "ELEMENT ENGINEERING FORCES FOR ELEMENT TYPE QUAD4"
    ]
  },
);

impl Display for BlockType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", self.desc());
  }
}
