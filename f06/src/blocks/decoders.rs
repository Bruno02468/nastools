//! This module implements the specific decoders for known data block types.

use std::collections::BTreeMap;

use log::*;

use crate::prelude::*;
use crate::util::*;

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
  data: RowBlock<f64, GridPointRef, Dof, { Self::MATWIDTH }>
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
  data: RowBlock<f64, GridPointForceOrigin, Dof, { Self::MATWIDTH }>
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
        let i0 = nth_integer(line, 0).map(|x| x as usize);
        let i1 = nth_integer(line, 1).map(|x| x as usize);
        self.gpref = match (i0, i1) {
          (Some(0), Some(x)) => Some(x),
          (Some(x), _) => Some(x),
          _ => None
        }.map(|x| x.into());
        if line.contains("*TOTALS*") {
          return LineResponse::Useless;
        } else if line.contains("APP-LOAD") {
          ForceOrigin::Load
        } else if line.contains("F-OF-SPC") {
          ForceOrigin::SinglePointConstraint
        } else if line.contains("F-OF-MPC") {
          ForceOrigin::MultiPointConstraint
        } else {
          let eid = line_breakdown(line)
            .filter_map(|lf| {
              if let LineField::Integer(eid) = lf {
                return Some(eid as usize);
              } else {
                return None;
              }
            }).last();
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
  /// The flavour of F06 file we're decoding SPC forces for.
  flavour: Flavour,
  /// The displacement data.
  data: RowBlock<f64, GridPointRef, Dof, { Self::MATWIDTH }>
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

/// This decodes an applied forces (load vector) block.
pub(crate) struct AppliedForcesDecoder {
  /// The flavour of F06 file we're decoding displacements for.
  flavour: Flavour,
  /// The displacement data.
  data: RowBlock<f64, GridPointRef, Dof, { Self::MATWIDTH }>
}

impl BlockDecoder for AppliedForcesDecoder {
  type MatScalar = f64;
  type RowIndex = GridPointRef;
  type ColumnIndex = Dof;
  const MATWIDTH: usize = SIXDOF;
  const BLOCK_TYPE: BlockType = BlockType::AppliedForces;

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
    let dofs: [f64; Self::MATWIDTH] = if let Some(arr) = extract_reals(line) {
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

/// A decoder for the "stresses in quad elements" table.
pub(crate) struct QuadStressesDecoder {
  /// The flavour of solver we're decoding for.
  flavour: Flavour,
  /// The inner block of data.
  data: RowBlock<f64, ElementSidedPoint, PlateStressField, { Self::MATWIDTH }>,
  /// Current row reference.
  cur_row: Option<<Self as BlockDecoder>::RowIndex>,
  /// Element type, hinted by the header.
  etype: Option<ElementType>
}

impl BlockDecoder for QuadStressesDecoder {
  type MatScalar = f64;
  type RowIndex = ElementSidedPoint;
  type ColumnIndex = PlateStressField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::QuadStresses;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      data: RowBlock::new(PlateStressField::canonical_cols()),
      cur_row: None,
      etype: None
    };
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn good_header(&mut self, header: &str) -> bool {
    self.etype = nth_etype(header, 0);
    if header.contains("THERMAL") || header.contains("ELASTIC") {
      return false;
    }
    return true;
  }

  fn hint_last(&mut self, last: NasIndex) {
    if let NasIndex::ElementSidedPoint(esp) = last {
      self.cur_row = Some(esp);
    }
  }

  fn last_row_index(&self) -> Option<NasIndex> {
    return self.cur_row.map(|q| q.into());
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    // first, take eight floats. if there aren't any, we're toast.
    let cols: [f64; Self::MATWIDTH] = if let Some(arr) = lax_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    // okay, now we get the sided point.
    let fields = line_breakdown(line).collect::<Vec<_>>();
    let ints = fields.iter()
      .filter_map(|lf| {
        if let LineField::Integer(i) = lf { Some(i) } else { None }
      }).copied().collect::<Vec<_>>();
    match self.flavour.solver {
      Some(Solver::Mystran) => {
        if ints.is_empty() {
          // cont. line
          if let Some(ref mut ri) = self.cur_row {
            ri.flip_side();
          } else {
            warn!("cont line without row index at {}", line);
            return LineResponse::Abort;
          }
        } else {
          // line has row info
          let point = if line.contains("CENTER") {
            ElementPoint::Centroid
          } else if let Some(gid) = ints.last() {
            ElementPoint::Corner((*gid as usize).into())
          } else {
            warn!("no point at {}", line);
            return LineResponse::Abort;
          };
          let side = ElementSide::Bottom;
          let eid = if let Some(LineField::Integer(eid)) = fields.first() {
            *eid as usize
          } else if let Some(ri) = self.cur_row {
            ri.element.eid
          } else {
            warn!("no eid at {}", line);
            return LineResponse::Abort;
          };
          self.cur_row.replace(ElementSidedPoint {
            element: ElementRef { eid, etype: self.etype },
            point,
            side
          });
        }
      },
      Some(Solver::Simcenter) => {
        if ints.is_empty() {
          // cont. line
          if let Some(ref mut ri) = self.cur_row {
            ri.flip_side();
          } else {
            warn!("cont line without row index at {}", line);
            return LineResponse::Abort;
          }
        } else {
          // line has row info
          let point = if line.contains("CEN/4") {
            ElementPoint::Centroid
          } else if let Some(gid) = ints.last() {
            ElementPoint::Corner((*gid as usize).into())
          } else {
            warn!("no point at {}", line);
            return LineResponse::Abort;
          };
          let side = ElementSide::Bottom;
          let eid = if let Some(x) = ints.get(1) {
            *x as usize
          } else if let Some(ri) = self.cur_row {
            ri.element.eid
          } else {
            warn!("no eid at {}", line);
            return LineResponse::Abort;
          };
          self.cur_row.replace(ElementSidedPoint {
            element: ElementRef { eid, etype: self.etype },
            point,
            side
          });
        }
      },
      None => return LineResponse::BadFlavour,
    }
    if let Some(rid) = self.cur_row {
      self.data.insert_raw(rid, &cols);
      return LineResponse::Data;
    } else {
      warn!("found data but couldn't construct row index at {}", line);
      return LineResponse::Abort;
    }
  }
}

/// A decoder for the "strains in quad elements" table. It just uses the same
/// decoder, transparently.
pub(crate) struct QuadStrainsDecoder {
  /// Just use the same decoder.
  inner: QuadStressesDecoder
}

impl BlockDecoder for QuadStrainsDecoder {
  type MatScalar = f64;
  type RowIndex = ElementSidedPoint;
  type ColumnIndex = PlateStrainField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::QuadStrains;

  fn new(flavour: Flavour) -> Self {
    return Self { inner: QuadStressesDecoder::new(flavour) }
  }

  fn good_header(&mut self, header: &str) -> bool {
    return BlockDecoder::good_header(&mut self.inner, header);
  }

  fn hint_last(&mut self, last: NasIndex) {
    BlockDecoder::hint_last(&mut self.inner, last);
  }

  fn last_row_index(&self) -> Option<NasIndex> {
    return BlockDecoder::last_row_index(&self.inner);
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    let mut fb = self.inner.unwrap(subcase, line_range);
    fb.col_indexes = fb.col_indexes.into_iter()
      .map(|(ci, n)| {
        if let NasIndex::PlateStressField(qss) = ci {
          return (PlateStrainField::from(qss).into(), n);
        } else {
          panic!("bad col index in quadstress");
        }
      })
      .collect();
    fb.block_type = Self::BLOCK_TYPE;
    return fb;
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    return BlockDecoder::consume(&mut self.inner, line);
  }
}

/// Decoder for quad element engineering forces.
pub(crate) struct QuadForcesDecoder {
  /// The flavour of solver we're decoding for.
  flavour: Flavour,
  /// The inner block of data.
  data: RowBlock<f64, PointInElement, PlateForceField, { Self::MATWIDTH }>,
  /// Current row reference.
  cur_row: Option<<Self as BlockDecoder>::RowIndex>,
  /// Element type, hinted by the header.
  etype: Option<ElementType>,
  /// Does this block hold grid-IDs?
  has_grid_id: bool
}

impl BlockDecoder for QuadForcesDecoder {
  type MatScalar = f64;
  type RowIndex = PointInElement;
  type ColumnIndex = PlateForceField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::QuadForces;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      data: RowBlock::new(PlateForceField::canonical_cols()),
      cur_row: None,
      etype: None,
      has_grid_id: false
    };
  }

  fn good_header(&mut self, header: &str) -> bool {
    self.etype = nth_etype(header, 0);
    return true;
  }

  fn hint_last(&mut self, last: NasIndex) {
    if let NasIndex::PointInElement(esp) = last {
      self.cur_row = Some(esp);
    }
  }

  fn last_row_index(&self) -> Option<NasIndex> {
    return self.cur_row.map(|q| q.into());
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    if line.contains("GRID-ID") {
      self.has_grid_id = true;
      return LineResponse::Metadata;
    }
    // first, take eight floats. if there aren't any, we're toast.
    let cols: [f64; Self::MATWIDTH] = if let Some(arr) = extract_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    // get the row ID.
    let fields = line_breakdown(line).collect::<Vec<_>>();
    let ints = fields.iter()
      .filter_map(|lf| {
        if let LineField::Integer(i) = lf { Some(i) } else { None }
      }).copied().collect::<Vec<_>>();
    match self.flavour.solver {
      Some(Solver::Mystran) => {
        if let Some(eid) = ints.first() {
          self.cur_row.replace(PointInElement {
            element: ElementRef { eid: *eid as usize, etype: self.etype },
            point: ElementPoint::Centroid
          });
        } else {
          self.cur_row = None;
        }
      },
      Some(Solver::Simcenter) => {
        // line has row info
        let eid: usize;
        let point: ElementPoint;
        if self.has_grid_id {
          // line has grid id
          point = if line.contains("CEN/4") {
            ElementPoint::Centroid
          } else if let Some(gid) = ints.last() {
            ElementPoint::Corner((*gid as usize).into())
          } else {
            warn!("no point at {}", line);
            return LineResponse::Abort;
          };
          eid = if let Some(x) = ints.get(1) {
            *x as usize
          } else if let Some(ri) = self.cur_row {
            ri.element.eid
          } else {
            warn!("no eid at {}", line);
            return LineResponse::Abort;
          };
        } else {
          // line with no grid id. easier.
          point = ElementPoint::Centroid;
          eid = if let Some(x) = ints.last() {
            *x as usize
          } else {
            warn!("no eid at {}", line);
            return LineResponse::Abort;
          };
        }
        self.cur_row.replace(PointInElement {
          element: ElementRef { eid, etype: self.etype },
          point
        });
      },
      None => return LineResponse::BadFlavour
    };
    // if we got a row ID, insert.
    if let Some(rid) = self.cur_row {
      self.data.insert_raw(rid, &cols);
      return LineResponse::Data;
    } else {
      warn!("found data but couldn't construct row index at {}", line);
      return LineResponse::Abort;
    }
  }
}

/// Decoder for tri element engineering forces.
pub(crate) struct TriaForcesDecoder {
  /// The flavour of solver we're decoding for.
  flavour: Flavour,
  /// The inner block of data.
  data: RowBlock<f64, ElementRef, PlateForceField, { Self::MATWIDTH }>,
  /// Element type, hinted by the header.
  etype: Option<ElementType>
}

impl BlockDecoder for TriaForcesDecoder {
  type MatScalar = f64;
  type RowIndex = ElementRef;
  type ColumnIndex = PlateForceField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::TriaForces;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      data: RowBlock::new(PlateForceField::canonical_cols()),
      etype: None
    };
  }

  fn good_header(&mut self, header: &str) -> bool {
    self.etype = nth_etype(header, 0);
    return true;
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    let cols: [f64; Self::MATWIDTH] = if let Some(arr) = extract_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    if let Some(eid) = nth_integer(line, 0) {
      let ri = ElementRef { eid: eid as usize, etype: self.etype };
      self.data.insert_raw(ri, &cols);
      return LineResponse::Useless;
    } else {
      warn!("line had data but no eid");
      return LineResponse::Abort;
    }
  }
}

/// Decoder for ROD element engineering forces.
pub(crate) struct RodForcesDecoder {
  /// The inner block of data.
  data: RowBlock<f64, ElementRef, RodForceField, 2>
}

impl BlockDecoder for RodForcesDecoder {
  type MatScalar = f64;
  type RowIndex = ElementRef;
  type ColumnIndex = RodForceField;
  const MATWIDTH: usize = 2;
  const BLOCK_TYPE: BlockType = BlockType::RodForces;

  fn new(_flavour: Flavour) -> Self {
    return Self { data: RowBlock::new(RodForceField::canonical_cols()) };
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    let mut fields = line_breakdown(line);
    let mut found = 0;
    loop {
      let (a, b, c) = (fields.next(), fields.next(), fields.next());
      match (a, b, c) {
        (
          Some(LineField::Integer(eid)),
          Some(LineField::Real(x)),
          Some(LineField::Real(y))
        ) => {
          let ri = ElementRef {
            eid: eid as usize,
            etype: Some(ElementType::Rod)
          };
          self.data.insert_raw(ri, &[x, y]);
          found += 1;
        },
        _ => { break; }
      };
    };
    if found > 0 {
      return LineResponse::Data;
    } else {
      return LineResponse::Useless;
    }
  }
}

/// Decoder for bar engineering forces table.
pub(crate) struct BarForcesDecoder {
  /// The inner block of data.
  data: RowBlock<f64, ElementRef, BarForceField, 8>
}

impl BlockDecoder for BarForcesDecoder {
  type MatScalar = f64;
  type RowIndex = ElementRef;
  type ColumnIndex = BarForceField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::BarForces;

  fn new(_flavour: Flavour) -> Self {
    return Self {
      data: RowBlock::new(BarForceField::canonical_cols()),
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
    let cols: [f64; 8] = if let Some(arr) = extract_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    if let Some(eid) = nth_integer(line, 0) {
      let ri = ElementRef {
        eid: eid as usize,
        etype: Some(ElementType::Bar),
      };
      self.data.insert_raw(ri, &cols);
      return LineResponse::Data;
    } else {
      warn!("no eid on bar force data line!");
      return LineResponse::Abort;
    }
  }
}

/// Decoder for ELAS1 engineering force blocks.
pub struct Elas1ForcesDecoder {
  /// The inner data block.
  data: RowBlock<f64, ElementRef, SingleForce, { Self::MATWIDTH }>
}

impl BlockDecoder for Elas1ForcesDecoder {
  type MatScalar = f64;
  type RowIndex = ElementRef;
  type ColumnIndex = SingleForce;
  const MATWIDTH: usize = 1;
  const BLOCK_TYPE: BlockType = BlockType::Elas1Forces;

  fn new(_flavour: Flavour) -> Self {
    return Self { data: RowBlock::new(SingleForce::canonical_cols()) };
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    let mut fields = line_breakdown(line);
    let mut found = 0;
    loop {
      let (a, b) = (fields.next(), fields.next());
      match (a, b) {
        (Some(LineField::Integer(eid)), Some(LineField::Real(x))) => {
          let ri = ElementRef {
            eid: eid as usize,
            etype: Some(ElementType::Elas1)
          };
          self.data.insert_raw(ri, &[x]);
          found += 1;
        },
        _ => { break; }
      };
    };
    if found > 0 {
      return LineResponse::Data;
    } else {
      return LineResponse::Useless;
    }
  }
}

/// A decoder for triangular elements' stresses.
pub struct TriaStressesDecoder {
  /// The flavour of solver we're doing.
  flavour: Flavour,
  /// The data within.
  data: RowBlock<f64, ElementSidedPoint, PlateStressField, { Self::MATWIDTH }>,
  /// The current element ID.
  eid: Option<usize>,
  /// The element type (gleaned from the header).
  etype: Option<ElementType>
}

impl BlockDecoder for TriaStressesDecoder {
  type MatScalar = f64;
  type RowIndex = ElementSidedPoint;
  type ColumnIndex = PlateStressField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::TriaStresses;

  fn new(flavour: Flavour) -> Self {
    return Self {
      flavour,
      data: RowBlock::new(PlateStressField::canonical_cols()),
      eid: None,
      etype: None,
    }
  }

  fn good_header(&mut self, header: &str) -> bool {
    self.etype = nth_etype(header, 0);
    return true;
  }

  fn hint_last(&mut self, last: NasIndex) {
    if let NasIndex::ElementSidedPoint(esp) = last {
      self.etype = esp.element.etype;
      self.eid = Some(esp.element.eid);
    } else {
      panic!("bad header passed to hint_last");
    }
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    return self.data.finalise(Self::BLOCK_TYPE, subcase, line_range);
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    let vals: [f64; 8] = if let Some(arr) = lax_reals(line) {
      arr
    } else {
      return LineResponse::Useless;
    };
    let i0 = nth_natural(line, 0);
    let i1 = nth_natural(line, 1);
    self.eid = match (self.flavour.solver, self.eid) {
      (Some(Solver::Mystran), None) => i0,
      (Some(Solver::Mystran), Some(_)) => i0.or(self.eid),
      (Some(Solver::Simcenter), None) => i1,
      (Some(Solver::Simcenter), Some(_)) => i1.or(self.eid),
      (None, _) => return LineResponse::BadFlavour,
    };
    let esp = if let Some(eid) = self.eid {
      let element = ElementRef { eid, etype: self.etype };
      let side = if nth_natural(line, 0).is_none() {
        ElementSide::Top
      } else {
        ElementSide::Bottom
      };
      let point = ElementPoint::Anywhere;
      ElementSidedPoint { element, point, side }
    } else {
      warn!("no eid on data line on {}", line);
      return LineResponse::Abort;
    };
    self.data.insert_raw(esp, &vals);
    return LineResponse::Data;
  }
}

/// Decoder for triangular element strains -- same as triangular element
/// stresses, so it basically does nothing but invoke the inner decoder and
/// change some values upon finalisation.
pub(crate) struct TriaStrainsDecoder {
  /// Just use the same decoder.
  inner: TriaStressesDecoder
}

impl BlockDecoder for TriaStrainsDecoder {
  type MatScalar = f64;
  type RowIndex = ElementSidedPoint;
  type ColumnIndex = PlateStrainField;
  const MATWIDTH: usize = 8;
  const BLOCK_TYPE: BlockType = BlockType::TriaStrains;

  fn new(flavour: Flavour) -> Self {
    return Self { inner: TriaStressesDecoder::new(flavour) }
  }

  fn good_header(&mut self, header: &str) -> bool {
    return BlockDecoder::good_header(&mut self.inner, header);
  }

  fn hint_last(&mut self, last: NasIndex) {
    BlockDecoder::hint_last(&mut self.inner, last);
  }

  fn last_row_index(&self) -> Option<NasIndex> {
    return BlockDecoder::last_row_index(&self.inner);
  }

  fn unwrap(
    self,
    subcase: usize,
    line_range: Option<(usize, usize)>
  ) -> FinalBlock {
    let mut fb = self.inner.unwrap(subcase, line_range);
    fb.col_indexes = fb.col_indexes.into_iter()
      .map(|(ci, n)| {
        if let NasIndex::PlateStressField(qss) = ci {
          return (PlateStrainField::from(qss).into(), n);
        } else {
          panic!("bad col index in quadstress");
        }
      })
      .collect();
    fb.block_type = Self::BLOCK_TYPE;
    return fb;
  }

  fn consume(&mut self, line: &str) -> LineResponse {
    return BlockDecoder::consume(&mut self.inner, line);
  }
}
