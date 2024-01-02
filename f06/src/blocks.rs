//! This submodule defines the blocks that make up an F06 file.

use std::collections::BTreeMap;

use nalgebra::{Matrix, Const, VecStorage, Dyn, RealField};
use serde::{Serialize, Deserialize};

use crate::fields::indexing::IndexType;

/// The default number of rows to allocate when creating a matrix.
const DEFAULT_PREALLOC_ROWS: usize = 64;

/// The default amount for reallocation in case the current number of rows is
/// exceeded.
const DEFAULT_REALLOC_ROWS: usize = 4*DEFAULT_PREALLOC_ROWS;

/// A block that contains an indexing type, some details, and a data matrix.
/// The number of columns is fixed -- F06 data don't grow horizontally. Types:
///   - S: the scalar type for the data within.
///   - R: the type of abstract index for the rows.
///   - C: the type of abstract index for the columns.
///   - W: the width of the matrix, a constant.
#[derive(Clone, Serialize, Deserialize)]
pub struct RowBlock<S: RealField, R: IndexType, C: IndexType, const W: usize> {
  /// The row indexes.
  row_indexes: BTreeMap<R, usize>,
  /// The column indexes.
  col_indexes: BTreeMap<C, usize>,
  /// The data within.
  data: Matrix<S, Dyn, Const<W>, VecStorage<S, Dyn, Const<W>>>
}

impl<S, R, C, const W: usize> RowBlock<S, R, C, W>
  where S: RealField + Default, R: IndexType, C: IndexType {
  /// Creates a new RowBlock with a set width and a pre-allocated size.
  pub fn new(col_indexes: BTreeMap<C, usize>, rows: Option<usize>) -> Self {
    let row_indexes: BTreeMap<R, usize> = BTreeMap::new();
    let nrows = rows.unwrap_or(DEFAULT_PREALLOC_ROWS);
    let data = Matrix::<
      S, Dyn, Const<W>, VecStorage<S, Dyn, Const<W>>
    >::zeros(nrows);
    return Self { row_indexes, col_indexes, data }
  }

  /// Returns the number of allocated rows.
  pub fn allocated_rows(&self) -> usize {
    return self.data.shape().0;
  }
}
