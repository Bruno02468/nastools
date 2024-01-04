//! This module implements a list of known data blocks and information related
//! to them, such as names for detection and decoder instantiation subroutines.

use serde::{Serialize, Deserialize};

use crate::blocks::OpaqueDecoder;
use crate::flavour::{SolType, Solver};

/// This contains all the known data blocks.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockType {
  /// Grid point displacements.
  Displacements,
  /// Single-point constraint forces.
  SpcForces
}

impl BlockType {
  /// Instantiates the decoder for this data block type.
  pub fn init_decoder(
    &self,
    _solver: Solver,
    _soltype: SolType
  ) -> Box<dyn OpaqueDecoder> {
    todo!()
  }
}
