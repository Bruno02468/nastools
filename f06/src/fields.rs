//! This module defines structures and code to represent the differnent fields
//! that can be found in Nastran output.
//! 
//! I break F06s in blocks, blocks into rows, rows into columns, fields are
//! indexed by row and column, stored as nalgebra matrices or vectors.
//! 
//! A mapping is then made between indexable types and the underlying matrices.
//! For instance, if a block's characteristic is:
//!   - rows: grid point + force origin
//!   - columns: degrees of freedom
//! A mapping will be made for every pair of abstract indexes and "real" matrix
//! indexes. I'll expand more on this later.

pub mod indexing;

use serde::{Serialize, de::DeserializeOwned};

/// The size of a small fixed field, in bytes.
pub const SMALL_FIELD_BYTES: usize = 8;

/// The size of a large fixed field, in bytes.
pub const LARGE_FIELD_BYTES: usize = 2*SMALL_FIELD_BYTES;


/// All field indexing types must implement this trait.
pub trait IndexType: Copy + Ord + Eq + Serialize + DeserializeOwned {
  /// The name of this type of index, all caps.
  const INDEX_NAME: &'static str;
}
