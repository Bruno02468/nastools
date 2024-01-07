//! This module implements the specific decoders for known data block types.

use std::collections::BTreeMap;

use crate::blocks::{BlockDecoder, RowBlock, LineResponse, FinalBlock};
use crate::blocks::indexing::GridPointRef;
use crate::blocks::types::BlockType;
use crate::flavour::Flavour;
use crate::geometry::{Dof, SIXDOF};
use crate::util::{line_breakdown, LineField};

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

  fn unwrap(self, subcase: usize) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    if line.contains("-------------") {
      return LineResponse::Done;
    }
    let fields = line_breakdown(line);
    let mut gid: Option<usize> = None;
    let mut dofs: [f64; SIXDOF] = [0.0; SIXDOF];
    let mut found_dof = 0;
    for field in fields {
      match field {
        LineField::Integer(i) if i > 0 => {
          match gid {
            Some(_) => continue,
            None => gid = Some(i as usize),
          }
        },
        LineField::Real(x) if found_dof < SIXDOF => {
          dofs[found_dof] = x;
          found_dof += 1;
        },
        LineField::Real(_) if found_dof == SIXDOF => {
          return LineResponse::Unsupported;
        },
        _ => continue
      }
    }
    match (gid, found_dof) {
      (Some(g), SIXDOF) => {
        self.data.insert_raw(g.into(), &dofs);
        return LineResponse::Data;
      },
      (_, _) => return LineResponse::Useless
    };
  }
}
