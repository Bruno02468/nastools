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
      pub fn spaceds(&self) -> &'static [&'static str] {
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
    ["D I S P L A C E M E N T S", "D I S P L A C E M E N T   V E C T O R"]
  },
  {
    "Grid point force balance",
    GridPointForceBalance,
    GridPointForceBalanceDecoder,
    ["G R I D   P O I N T   F O R C E   B A L A N C E"]
  },
  {
    "Forces of single-point constraint",
    SpcForces,
    SpcForcesDecoder,
    [
      "S P C   F O R C E S",
      "F O R C E S   O F   S I N G L E - P O I N T   C O N S T R A I N T"
    ]
  },
);

impl Display for BlockType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", self.desc());
  }
}
