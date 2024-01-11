//! This module implements the generic parser for F06 files, and associated
//! structures and enums.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, BufReader, BufRead};
use std::path::Path;

use log::{debug, error, warn};
use serde::{Serialize, Deserialize};

use crate::prelude::*;
use crate::util::*;

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
  /// The line contained a warning.
  Warning,
  /// The line contained a fatal.
  Fatal,
  /// The line told us whihc kind of solution we're looking at.
  SolType(SolType),
  /// The line told us to start decoding a block.
  BeginBlock(BlockType),
  /// The line was passed to a block decoder.
  PassedToDecoder(BlockType, LineResponse),
  /// The line was a block beginning, but I didn't instantiate a block decoder
  /// because we don't even know the solver yet!
  BeginningWithoutSolver,
  /// This line indicates the beginning of a block we don't even know yet.
  PotentialHeader
}

/// This is the F06 parser -- it doesn't care how lines are fed into it.
/// It's one-pass, single-thread. There might be a parallel one later.
pub struct OnePassParser {
  /// The current file.
  file: F06File,
  /// The current subcase.
  subcase: usize,
  /// The decoder for block we're currently in.
  current_decoder: Option<Box<dyn OpaqueDecoder>>,
  /// The total number of consumed lines.
  total_lines: usize,
  /// Line of the last block beginning.
  last_block_start: usize
}

impl Default for OnePassParser {
  fn default() -> Self {
    return Self::new();
  }
}

impl OnePassParser {
  /// Instantiates a new parser.
  pub fn new() -> Self {
    return Self {
      file: F06File::new(),
      subcase: 1,
      current_decoder: None,
      total_lines: 0,
      last_block_start: 0
    };
  }

  /// Hints the parser about the flavour.
  pub fn hint_flavour(&mut self, flavour: Flavour) {
    self.file.flavour.solver = self.file.flavour.solver.or(flavour.solver);
    self.file.flavour.soltype = self.file.flavour.soltype.or(flavour.soltype);
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
    let mut candidates = BlockType::all()
      .iter()
      .copied()
      .filter(|bt| bt.spaceds().iter().any(|s| line.contains(s)))
      .collect::<BTreeSet<_>>();
    match candidates.len() {
      0 => return None,
      1 => return Some(candidates.pop_first().unwrap()),
      _ => warn!(
        "Line {} matches more than one block type!", self.total_lines
      )
    }
    return None;
  }

  /// Flushes the current block decoder into the file.
  fn flush_decoder(&mut self) {
    if let Some(dec) = self.current_decoder.take() {
      debug!(
        "Finishing up a \"{}\" block on line {}.",
        dec.block_type(),
        self.total_lines
      );
      let line_range = Some((self.last_block_start, self.total_lines+1));
      let fb = dec.finalise(self.subcase, line_range);
      if !fb.row_indexes.is_empty() {
        self.file.insert_block(fb);
      }
    }
  }

  /// Consumes a line into the parser.
  pub fn consume(&mut self, line: &str) -> ParserResponse {
    self.total_lines += 1;
    // first, try and enhance our knowledge of the flavour from the line.
    if let Some(solver) = self.detect_solver(line) {
      self.file.flavour.solver = Some(solver);
      debug!("Line {} told us the solver is {}!", self.total_lines, solver);
      return ParserResponse::Solver(solver);
    }
    // check for a subcase change
    if let Some(subcase) = self.detect_subcase(line) {
      if self.subcase != subcase {
        // a subcase change definitely means we should stop the block
        self.flush_decoder();
        debug!(
          "Switched from subcase {} to {} on line {}!",
          self.subcase,
          subcase,
          self.total_lines
        );
        self.subcase = subcase;
      }
      return ParserResponse::Subcase(subcase);
    }
    // check for warning
    if line.contains("WARNING") {
      debug!("Found warning on line {}: {}", self.total_lines, line);
      self.file.warnings.insert(self.total_lines, line.to_string());
      return ParserResponse::Warning;
    }
    // check for fatal
    if line.contains("FATAL") {
      debug!("Found fatal on line {}: {}", self.total_lines, line);
      self.file.fatal_errors.insert(self.total_lines, line.to_string());
      return ParserResponse::Fatal;
    }
    // now, check for a block beginning.
    if let Some(bt) = self.detect_beginning(line) {
      if self.file.flavour.solver.is_none() {
        error!(
          "Found a block start on line {} before knowing the solver!",
          self.total_lines
        );
        return ParserResponse::BeginningWithoutSolver;
      }
      // this is a block beginning. flush the current decoder.
      self.flush_decoder();
      // start a new decoder.
      self.last_block_start = self.total_lines;
      debug!("Started a \"{}\" block on line {}!", bt, self.total_lines);
      self.current_decoder = Some(bt.init_decoder(self.file.flavour));
      return ParserResponse::BeginBlock(bt);
    }
    // check for a potential header.
    if let Some(s) = check_header(line) {
      self.file.potential_headers.insert(PotentialHeader {
        start: self.total_lines,
        span: 1,
        text: s,
      });
      debug!(
        "Found a potential header in line {}! Flushing active decoder.",
        self.total_lines
      );
      self.flush_decoder();
    }
    // well, is there a current block decoder? if so, pass it the line.
    if let Some(ref mut dec) = self.current_decoder {
      let resp = dec.consume(line);
      let bt = dec.block_type();
      if resp.abnormal() {
        self.flush_decoder();
      }
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

  /// Parses from a BufRead instance.
  pub fn parse_bufread<R: BufRead>(reader: R) -> io::Result<F06File> {
    let mut parser = Self::new();
    for line in reader.lines() {
      match parser.consume(&line?) {
        ParserResponse::PassedToDecoder(bt, lr) if lr.abnormal() => debug!(
          "Got abnormal response {:?} from {} while parsing line {}!",
          lr,
          bt,
          parser.total_lines
        ),
        ParserResponse::BeginningWithoutSolver => debug!(
          "Found block beginning in line {} before detecting the solver!",
          parser.total_lines
        ),
        _ => {}
      }
    }
    return Ok(parser.finish());
  }

  /// Utility method -- reads and parses a file.
  pub fn parse_file<S: AsRef<Path>>(p: S) -> io::Result<F06File> {
    let file = File::open(p.as_ref())?;
    let mut f06 = Self::parse_bufread(BufReader::new(file))?;
    f06.filename = p.as_ref().file_name()
      .and_then(|s| s.to_str())
      .map(String::from);
    return Ok(f06);
  }
}
