//! Dumps an F06 file into a JSON.

#![allow(clippy::needless_return)] // i'll never forgive rust for this
#![allow(dead_code)] // temporary

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

use f06::blocks::indexing::{GridPointRef, NasIndex};
use f06::blocks::types::BlockType;
use f06::parser::OnePassParser;

fn main() -> io::Result<()> {
  let mut parser = OnePassParser::new();
  let fp = concat!(
    r"C:\Users\bruno\Dropbox\mystran_work\benchmark\F06\",
    r"SB-ALL-ELEM-TEST.F06"
  );
  let file = File::open(fp)?;
  for line in BufReader::new(file).lines() {
    parser.consume(&line?);
  }
  let f06 = parser.finish();
  let gid = 1011;
  let gpref = GridPointRef::from(gid);
  let subcase = 91;
  let gpfb = f06.blocks.iter()
    .filter(|b| b.block_type == BlockType::GridPointForceBalance)
    .find(|b| b.subcase == subcase)
    .expect("couldn't find grid point force balance for subcase 91");
  println!("Force balance for grid point {} in subcase {}:", gid, subcase);
  for ri in gpfb.row_indexes.keys() {
    if let NasIndex::GridPointForceOrigin(gpfo) = ri {
      if gpfo.grid_point == gpref {
        print!("  {}: ", gpfo.force_origin);
        for ci in gpfb.col_indexes.keys() {
          print!("{}  ", gpfb.get(*ri, *ci).expect("bad indexes"));
        }
        println!();
      }
    }
  }
  return Ok(());
}
