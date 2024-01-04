//! This submodule defines the blocks that make up an F06 file.

pub mod known;
mod decoders;

use std::collections::BTreeMap;

use nalgebra::{Matrix, Const, VecStorage, Dyn, Scalar};
use num::Zero;
use serde::{Serialize, Deserialize};

use crate::fields::indexing::{IndexType, NasIndex};

/// This trait encapsulates the necessary properties for a scalar that can exist
/// in the data matrices.
pub trait NasScalar: Copy + Scalar + Zero {}

/// This type encapsulates a dynamic matrix of scalar type S and width W.
pub type DynMatx<S, const W: usize> = Matrix<
  S, Dyn, Const<W>, VecStorage<S, Dyn, Const<W>>
>;

/// A block that contains an indexing type, some details, and a data matrix.
/// The number of columns is fixed -- F06 data don't grow horizontally. Types:
///   - S: the scalar type for the data within.
///   - R: the type of abstract index for the rows.
///   - C: the type of abstract index for the columns.
///   - W: the width of the matrix, a constant.
#[derive(Clone, Serialize, Deserialize)]
pub struct RowBlock<S: NasScalar, R: IndexType, C: IndexType, const W: usize> {
  /// The row indexes.
  row_indexes: BTreeMap<R, usize>,
  /// The column indexes.
  col_indexes: BTreeMap<C, usize>,
  /// The data within.
  data: Option<DynMatx<S, W>>
}

impl<S, R, C, const W: usize> RowBlock<S, R, C, W>
  where S: NasScalar, R: IndexType, C: IndexType {
  /// Creates a new RowBlock with a set width and a pre-allocated size.
  pub(crate) fn new(col_indexes: BTreeMap<C, usize>) -> Self {
    let row_indexes: BTreeMap<R, usize> = BTreeMap::new();
    return Self { row_indexes, col_indexes, data: None }
  }
  
  /// Inserts a line raw into the data matrix, without fixing indexes. Returns
  /// the row within the underlying matrixes this was put in.
  pub(crate) fn insert_row_raw(&mut self, row_index: R, row: &[S; W]) -> usize {
    let irow: usize;
    if let Some(mut mat) = self.data.take() {
      if let Some(fnd) = self.row_indexes.get(&row_index) {
        irow = *fnd;
      } else {
        irow = mat.shape().0;
        mat = mat.insert_row(irow, S::zero());
      }
      mat.row_mut(irow).copy_from_slice(row);
      self.data = Some(mat);
    } else {
      irow = 0;
      let mat = DynMatx::<S, W>::from_row_slice(row);
      self.data = Some(mat);
    }
    self.row_indexes.insert(row_index, irow);
    return irow;
  }

  /// Returns the column indexes as known by the RowBlock. This way, you can
  /// set up your slices adequately and pass them directly into insert_row_raw,
  /// which is much, much faster.
  pub(crate) fn col_indexes(&self) -> &BTreeMap<C, usize> {
    return &self.col_indexes;
  } 
  
  /// Returns the row indexes.
  pub(crate) fn row_indexes(&self) -> &BTreeMap<R, usize> {
    return &self.row_indexes;
  }

  /// Inserts a line into the data matrix, but passing column indexes because
  /// you might not know the underlying column order.
  pub(crate) fn insert_row(
    &mut self,
    row_index: R,
    data: &BTreeMap<C, S>
  ) -> usize {
    let mut raw_data = [S::zero(); W];
    data.iter().for_each(|(c, s)| {
      let ri = self.col_indexes.get(c).expect("bad col index");
      raw_data[*ri] = *s;
    });
    return self.insert_row_raw(row_index, &raw_data);
  }

  /// Returns a reference to an item in the matrix.
  pub fn get(&self, row: &R, col: &C) -> Option<&S> {
    if let Some(i) = self.row_indexes.get(row) {
      if let Some(j) = self.col_indexes.get(col) {
        if let Some(ref matx) = self.data {
          return matx.get((*i, *j));
        }
      }
    }
    return None;
  }

  /// Returns a mutable view into an item in the matrix.
  pub fn get_mut(&mut self, row: &R, col: &C) -> Option<&mut S> {
    if let Some(i) = self.row_indexes.get(row) {
      if let Some(j) = self.col_indexes.get(col) {
        if let Some(ref mut matx) = self.data {
          return matx.get_mut((*i, *j));
        }
      }
    }
    return None;
  }

  /// Consumes this data structure and unwraps the matrix within. This is to
  /// avoid inconsistent data ever residing within this block.
  pub fn unwrap_matrix(self) -> Option<DynMatx<S, W>> {
    return self.data;
  }
}

/// Response of a block parser upon receiving a line.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum LineResponse {
  /// The supplied line contained no useful information.
  Useless,
  /// The supplied line contained useful metadata, but was not a data line.
  Metadata,
  /// The supplied line contained useful data I've put into the matrix.
  Data,
  /// The supplied line makes me think this block is finished.
  Done,
  /// The supplied line looks like it contains data but I lack the necessary
  /// metadata to decode it.
  MissingMetadata,
  /// The supplied line makes me think I'm the wrong decoder but right solver.
  WrongDecoder,
  /// The supplied line makes me think you got the solver wrong.
  WrongSolver,
  /// Something's done terribly wrong.
  Abort
}

/// This trait is implemented by all known output block decoders. It aids with
/// code uniformity.
pub(crate) trait BlockDecoder {
  /// The type of scalar in the data matrix.
  type MatScalar: NasScalar;
  /// The type of row index.
  type RowIndex: IndexType;
  /// The type of column index.
  type ColumnIndex: IndexType;
  /// The width of the actual data matrix -- doesn't count indexes.
  const MATWIDTH: usize;

  /// Inserts a row into the underlying data matrix.
  fn insert_row(&mut self, data: Vec<Self::MatScalar>);
}

/// This trait is used to hide implementation details of a block decoder.
pub trait OpaqueDecoder {
  /// This function takes in a line and loads it into the decoder.
  fn consume(
    &mut self,
    line: &str
  ) -> LineResponse;

  /// Returns a vector with the row indexes.
  fn row_indexes(&self) -> Vec<NasIndex>;

  /// Returns a vector with the column indexes.
  fn col_indexes(&self) -> Vec<NasIndex>;
}
