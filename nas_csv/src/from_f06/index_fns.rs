//! Contains several functions to extract fields from NasIndex variants.

use f06::prelude::*;

use crate::prelude::*;

/// Attempts to extract a grid point ID from an index type.
pub fn ixfn_gid(index: NasIndex) -> Result<CsvField, ConversionError> {
  let bad_col_type = Err(ConversionError::BadColIndexType(index));
  return Ok(match index {
    NasIndex::GridPointRef(g) => g.gid,
    NasIndex::PointInElement(pie) => match pie.point {
      ElementPoint::Corner(g) => g.gid,
      _ => return bad_col_type
    },
    NasIndex::GridPointForceOrigin(gpfo) => gpfo.grid_point.gid,
    NasIndex::ElementSidedPoint(esp) => match esp.point {
      ElementPoint::Corner(g) => g.gid,
      _ => return bad_col_type
    },
    _ => return bad_col_type
  }.into());
}

/// Utility functions: extracts element references from index types.
fn util_eref(index: NasIndex) -> Result<ElementRef, ConversionError> {
  let bad_col_type = Err(ConversionError::BadColIndexType(index));
  return Ok(match index {
    NasIndex::ElementRef(eref) => eref,
    NasIndex::PointInElement(pie) => pie.element,
    NasIndex::GridPointForceOrigin(gpfo) => match gpfo.force_origin {
      ForceOrigin::Element { elem } => elem,
      _ => return bad_col_type
    },
    NasIndex::ElementSidedPoint(esp) => esp.element,
    _ => return bad_col_type
  });
}

/// Attempts to extract an element ID from an index type.
pub fn ixfn_eid(index: NasIndex) -> Result<CsvField, ConversionError> {
  return util_eref(index).map(|eref| eref.eid.into());
}

/// Attempts to extract an element type from an index type.
pub fn ixfn_etype(index: NasIndex) -> Result<CsvField, ConversionError> {
  if let Some(etype) = util_eref(index).map(|eref| eref.etype)? {
    return Ok(etype.into());
  } else {
    return Ok("<UNKNOWN>".to_owned().into());
  }
}
