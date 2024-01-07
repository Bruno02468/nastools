//! This module implements the generic parser for F06 files, and associated
//! structures and enums.

use serde::{Serialize, Deserialize};

use crate::blocks::{OpaqueDecoder, LineResponse};
use crate::blocks::types::BlockType;
use crate::f06file::F06File;
use crate::flavour::{ Solver, SolType};
use crate::util::{line_breakdown, LineField};

/// A parser might respond this when successfully decoding a line.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ParserResponse {
  /// The line was useless.
  Useless,
  /// The line helped us learn more about the solver.
  Solver(Solver),
  /// This line told us the current subcase.
  Subcase(usize),
  /// The line told us whihc kind of solution we're looking at.
  SolType(SolType),
  /// The line told us to start decoding a block.
  BeginBlock(BlockType),
  /// The line was passed to a block decoder.
  PassedToDecoder(BlockType, LineResponse)
}

/// This is the F06 parser -- it doesn't care how lines are fed into it.
/// It's one-pass, single-thread. There might be a parallel one later.
pub struct OnePassParser {
  /// The current file.
  file: F06File,
  /// The current subcase.
  subcase: usize,
  /// The decoder for block we're currently in.
  current_decoder: Option<Box<dyn OpaqueDecoder>>
}

impl Default for OnePassParser {
  fn default() -> Self {
    return Self::new();
  }
}

impl OnePassParser {
  /// Instantiates a new parser.
  pub fn new() -> Self {
    return Self { file: F06File::new(), subcase: 1, current_decoder: None };
  }

  /// Tries to update the solver in based on a line.
  fn detect_solver(&self, line: &str) -> Option<Solver> {
    if self.file.flavour.solver.is_none() {
      for cand in Solver::all() {
        if line.contains(cand.name()) {
          return Some(*cand);
        }
      }
    }
    return None;
  }

  /// Tries to detect a change in subcase.
  fn detect_subcase(&self, line: &str) -> Option<usize> {
    if line.contains("OUTPUT FOR SUBCASE") {
      return line_breakdown(line)
        .filter_map(|field| {
          if let LineField::Integer(x) = field {
            return Some(x as usize)
          } else {
            None
          }
      }).nth(0);
    }
    return None;
  }

  /// Checks if a line signals a block beginning.
  fn detect_beginning(&self, line: &str) -> Option<BlockType> {
    for bt in BlockType::all() {
      for spaced in bt.spaceds() {
        if line.contains(spaced) {
          return Some(*bt);
        }
      }
    }
    return None;
  }

  /// Flushes the current block decoder into the file.
  fn flush_decoder(&mut self) {
    if let Some(dec) = self.current_decoder.take() {
      let fb = dec.finalise(self.subcase);
      if !fb.row_indexes.is_empty() {
        self.file.blocks.push(fb);
      }
    }
  }

  /// Consumes a line into the parser.
  pub fn consume(&mut self, line: &str) -> ParserResponse {
    // first, try and enhance our knowledge of the flavour from the line.
    if let Some(solver) = self.detect_solver(line) {
      self.file.flavour.solver = Some(solver);
      return ParserResponse::Solver(solver);
    }
    // check for a subcase change
    if let Some(subcase) = self.detect_subcase(line) {
      self.subcase = subcase;
      return ParserResponse::Subcase(subcase);
    }
    // now, check for a block beginning.
    if let Some(bt) = self.detect_beginning(line) {
      // this is a block beginning. flush the current decoder.
      self.flush_decoder();
      // start a new decoder.
      self.current_decoder = Some(bt.init_decoder(self.file.flavour));
      return ParserResponse::BeginBlock(bt);
    }
    // well, is there a current block decoder? if so, pass it the line.
    if let Some(ref mut dec) = self.current_decoder {
      let resp = dec.consume(line);
      let bt = dec.block_type();
      match resp {
        LineResponse::Done
        | LineResponse::BadFlavour
        | LineResponse::MissingMetadata
        | LineResponse::WrongDecoder
        | LineResponse::WrongSolver
        | LineResponse::Abort => self.flush_decoder(),
        _ => ()
      };
      return ParserResponse::PassedToDecoder(bt, resp);
    }
    // well, the line was useless then.
    return ParserResponse::Useless;
  }

  /// Finishes up and returns the file struct.
  pub fn finish(mut self) -> F06File {
    self.flush_decoder();
    return self.file;
  }
}
