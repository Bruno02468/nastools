//! This module implements tools and structures to use when comparing F06 files
//! (especially meant for the `f06diff` tool).

use serde::{Serialize, Deserialize};

//use crate::prelude::*;



/// This structure holds the differences found between two F06Files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct F06Diff {
  // Pairs of blocks that were compared and their diffs.
  //pub compared_blocks: Vec<(&'f FinalBlock, &'s FinalBlock, )>
}
