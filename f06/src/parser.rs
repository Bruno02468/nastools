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
  /// This line contained part of a block header, we're yet to figure out if it
  /// corresponds to a known block or not,
  BlockHeader,
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
  last_block_start: usize,
  /// Accumulator of block header strings.
  header_accumulator: Vec<String>
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
      last_block_start: 0,
      header_accumulator: Vec::new()
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

  /// Flushes the current block header accumulator.
  fn flush_header(&mut self) -> Option<(String, usize)> {
    if self.header_accumulator.is_empty() {
      return None;
    } else {
      let num = self.header_accumulator.len();
      let full_name = self.header_accumulator.join(" ");
      self.header_accumulator.clear();
      return Some((full_name, num));
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
      self.flush_header();
      return ParserResponse::Subcase(subcase);
    }
    // check for warning
    if line.contains("WARNING") {
      debug!("Found warning on line {}: {}", self.total_lines, line);
      self.file.warnings.insert(self.total_lines, line.to_string());
      self.flush_header();
      return ParserResponse::Warning;
    }
    // check for fatal
    if line.contains("FATAL") {
      debug!("Found fatal on line {}: {}", self.total_lines, line);
      self.file.fatal_errors.insert(self.total_lines, line.to_string());
      self.flush_header();
      return ParserResponse::Fatal;
    }
    // check for a block header part.
    if let Some(unspaced) = check_header(line) {
      self.header_accumulator.push(unspaced);
      return ParserResponse::BlockHeader;
    } else if let Some((full_name, num_lines)) = self.flush_header() {
      // not a block header, but we were accumulating one.
      // first, flush the current decoder.
      self.flush_decoder();
      // is it the header of a known block?
      let mut candidates = BlockType::all()
        .iter()
        .copied()
        .filter(|bt| bt.headers().iter().any(|s| full_name.contains(s)))
        .collect::<BTreeSet<_>>();
      match candidates.len() {
        0 => {
          // not a known block. push a potential header.
          self.file.potential_headers.insert(PotentialHeader {
            start: self.total_lines-num_lines,
            span: num_lines,
            text: full_name,
          });
          debug!(
            "Found a potential header ending in line {}! Flushing.",
            self.total_lines
          );
          return ParserResponse::PotentialHeader;
        },
        1 => {
          let bt = candidates.pop_first().unwrap();
          // do we know the solver?
          if self.file.flavour.solver.is_none() {
            // nope
            error!(
              "Found a block start on line {} before knowing the solver!",
              self.total_lines
            );
            return ParserResponse::BeginningWithoutSolver;
          } else {
            // ok, begin the block then.
            let mut dec = bt.init_decoder(self.file.flavour);
            if dec.good_header(&full_name) {
              debug!("Started a \"{}\" block on line {}!", bt, self.total_lines);
              self.last_block_start = self.total_lines;
              self.current_decoder = Some(dec);
            } else {
              // bad header, whoops.
              self.file.potential_headers.insert(PotentialHeader {
                start: self.total_lines-num_lines,
                span: num_lines,
                text: full_name,
              });
              debug!(
                "Found a potential header ending in line {}! Flushing.",
                self.total_lines
              );
              return ParserResponse::PotentialHeader;
            }
          }
        },
        _ => warn!(
          "Line {} matches more than one block type!",
          self.total_lines
        )
      }
    }
    // if we got here, the line NOT a block header, and if there was a header
    // being accumulated, it was flushed and the decoder is active.
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
        ParserResponse::PassedToDecoder(bt, lr) if lr.abnormal() => warn!(
          "Got abnormal response {:?} from {} while parsing line {}!",
          lr,
          bt,
          parser.total_lines
        ),
        ParserResponse::BeginningWithoutSolver => warn!(
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
