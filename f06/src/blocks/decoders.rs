//! This module implements the specific decoders for known data block types.

use std::collections::BTreeMap;

use crate::blocks::*;
use crate::blocks::indexing::*;
use crate::blocks::types::BlockType;
use crate::flavour::{Flavour, Solver};
use crate::geometry::{Dof, SIXDOF};
use crate::util::*;

/// Dashes that signal the end of a table in MYSTRAN.
const MYSTRAN_DASHES: &str = "-------------";

/// Returns column indexes for DOFs. Used by a lot of things.
fn dof_cols() -> BTreeMap<Dof, usize> {
  return Dof::all()
    .iter()
    .copied()
    .enumerate()
    .map(|(a, b)| (b, a))
    .collect();
}

/// This decodes a displacements block.
pub(crate) struct DisplacementsDecoder {
  /// The flavour of F06 file we're decoding displacements for.
  flavour: Flavour,
  /// The displacement data.
  data: RowBlock<f64, GridPointRef, Dof, SIXDOF>
}

impl BlockDecoder for DisplacementsDecoder {
  type MatScalar = f64;
  type RowIndex = GridPointRef;
  type ColumnIndex = Dof;
  const MATWIDTH: usize = SIXDOF;
  const BLOCK_TYPE: BlockType = BlockType::Displacements;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      data: RowBlock::new(dof_cols())
    };
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    if line.contains(MYSTRAN_DASHES) {
      return LineResponse::Done;
    }
    let dofs: [f64; SIXDOF] = if let Some(arr) = extract_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    if let Some(gid) = nth_integer(line, 0) {
      self.data.insert_raw((gid as usize).into(), &dofs);
      return LineResponse::Data;
    }
    return LineResponse::Useless;
  }
}

/// The decoder for grid point force balance blocks.
pub(crate) struct GridPointForceBalanceDecoder {
  /// The flavour of F06 file we're decoding displacements for.
  flavour: Flavour,
  /// The current grid point ID.
  gpref: Option<GridPointRef>,
  /// The force balance data.
  data: RowBlock<f64, GridPointForceOrigin, Dof, SIXDOF>
}

impl BlockDecoder for GridPointForceBalanceDecoder {
  type MatScalar = f64;
  type RowIndex = GridPointForceOrigin;
  type ColumnIndex = Dof;
  const MATWIDTH: usize = SIXDOF;
  const BLOCK_TYPE: BlockType = BlockType::GridPointForceBalance;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      gpref: None,
      data: RowBlock::new(dof_cols()),
    };
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    if line.contains(MYSTRAN_DASHES) {
      return LineResponse::Done;
    }
    if line.contains("FORCE BALANCE FOR GRID POINT") {
      self.gpref = nth_integer(line, 0).map(|x| (x as usize).into());
      return LineResponse::Metadata;
    }
    if line.contains("*TOTALS*") {
      return LineResponse::Useless;
    }
    let fo: ForceOrigin = match self.flavour.solver {
      Some(Solver::Mystran) => {
        if line.contains("APPLIED FORCE") {
          ForceOrigin::Load
        } else if line.contains("SPC FORCE") {
          ForceOrigin::SinglePointConstraint
        } else if line.contains("MPC FORCE") {
          ForceOrigin::MultiPointConstraint
        } else if line.contains("ELEM") {
          if let Some(eid) = nth_integer(line, 0) {
            ForceOrigin::Element {
              elem: ElementRef {
                eid: eid as usize,
                etype: nth_etype(line, 0)
              }
            }
          } else {
            return LineResponse::Useless
          }
        } else {
          return LineResponse::Useless;
        }
      },
      Some(Solver::Simcenter) => {
        self.gpref = nth_integer(line, 0).map(|x| (x as usize).into());
        if line.contains("*TOTALS*") {
          return LineResponse::Useless;
        } else if line.contains("APP-LOAD") {
          self.gpref = nth_integer(line, 1).map(|x| (x as usize).into());
          ForceOrigin::Load
        } else if line.contains("F-OF-SPC") {
          ForceOrigin::SinglePointConstraint
        } else if line.contains("F-OF-MPC") {
          ForceOrigin::MultiPointConstraint
        } else {
          let eid = nth_integer(line, 1).map(|x| (x as usize));
          let etype_opt = nth_etype(line, 0);
          match (self.gpref, eid, etype_opt) {
            (Some(_), Some(eid), Some(etype)) => ForceOrigin::Element {
              elem: ElementRef { eid, etype: Some(etype) }
            },
            _ => return LineResponse::Useless
          }
        }
      },
      None => return LineResponse::BadFlavour
    };
    if let Some(gpref) = self.gpref {
      let ri = GridPointForceOrigin {
        grid_point: gpref,
        force_origin: fo,
      };
      if let Some(arr) = extract_reals::<SIXDOF>(line) {
        self.data.insert_raw(ri, &arr);
        return LineResponse::Data;
      } else {
        return LineResponse::BadFlavour;
      }
    }
    return LineResponse::Useless;
  }
}

/// Decoder for the SPC forces block type.
pub(crate) struct SpcForcesDecoder {
  /// The flavour of F06 file we're decoding displacements for.
  flavour: Flavour,
  /// The displacement data.
  data: RowBlock<f64, GridPointRef, Dof, SIXDOF>
}

impl BlockDecoder for SpcForcesDecoder {
  type MatScalar = f64;
  type RowIndex = GridPointRef;
  type ColumnIndex = Dof;
  const MATWIDTH: usize = SIXDOF;
  const BLOCK_TYPE: BlockType = BlockType::SpcForces;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      data: RowBlock::new(dof_cols())
    };
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    if line.contains(MYSTRAN_DASHES) {
      return LineResponse::Done;
    }
    let dofs: [f64; SIXDOF] = if let Some(arr) = extract_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    if let Some(gid) = nth_integer(line, 0) {
      self.data.insert_raw((gid as usize).into(), &dofs);
      return LineResponse::Data;
    }
    return LineResponse::Useless;
  }
}
