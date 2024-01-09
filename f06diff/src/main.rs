//! This utility loads two F06 files and tells you differences in detected
//! blocks, fields, etcetera. Obviously, you should pass the F06 files
//! generated from running the same model.
//!
//! The main purpose of this program is to compare two solvers running the same
//! model. This way, we can verify MYSTRAN more easily.

#![allow(clippy::needless_return)] // i'll never forgive rust for this

use std::io::{self, BufReader};
use std::path::PathBuf;

use clap::Parser;
use log::{LevelFilter, info, error};
use f06::prelude::*;

#[derive(Parser)]
#[command(author, version)]
struct Cli {
  /// Output extra/debug info while parsing.
  #[arg(short, long)]
  verbose: bool,
  /// Path to the first file.
  first: PathBuf,
  /// Path to the second file. Set to "-" to read from stdin.
  second: PathBuf
}

fn main() -> io::Result<()> {
  // init cli stuff
  let args = Cli::parse();
  let log_level = if args.verbose {
    LevelFilter::Debug
  } else {
    LevelFilter::Info
  };
  env_logger::builder().filter_level(log_level).init();
  // parse the first file
  let mut first: F06File = if args.first.is_file() {
    if let Some(bn) = args.first.file_name() {
      if let Some(sbn) = bn.to_str() {
        info!("Loading {}...", sbn);
      }
    } else {
      info!("Loading first file...");
    }
    OnePassParser::parse_file(&args.first)?
  } else {
    error!("Second path either does not exist or is not a file!");
    std::process::exit(1);
  };
  // parse the second file
  let mut second: F06File = if args.second.as_os_str().eq_ignore_ascii_case("-") {
    OnePassParser::parse_bufread(BufReader::new(io::stdin()))?
  } else if args.second.is_file() {
    if let Some(bn) = args.second.file_name() {
      if let Some(sbn) = bn.to_str() {
        info!("Loading {}...", sbn);
      }
    } else {
      info!("Loading first file...");
    }
    OnePassParser::parse_file(&args.second)?
  } else {
    error!("Second path either does not exist or is not a file!");
    std::process::exit(1);
  };
  // tidy stuff up
  for b in [&mut first, &mut second] {
    b.merge_blocks();
    b.merge_potential_headers();
    b.sort_all_blocks();
  }
  return Ok(());
}
