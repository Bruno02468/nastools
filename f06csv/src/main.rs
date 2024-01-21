//! A command-line application to convert Nastran F06 output to CSV.

#![allow(clippy::needless_return)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use log::*;
use f06::prelude::*;
use nas_csv::from_f06::templates::all_converters;
use nas_csv::prelude::*;

/// The arguments passed to the converter.
#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
struct Cli {
  /// CSV blocks to write. Can be specified more than once, or comma-separated.
  /// If absent, all blocks are written.
  #[arg(short = 'b', long = "blocks", num_args = 0.., value_delimiter = ',')]
  csv_blocks: Vec<CsvBlockId>,
  /// If a record has a grid point ID, only output those that contain the
  /// specified IDs. Can be specified more than once, or comma-separated.
  /// If absent, no filter is applied.
  #[arg(short = 'g', long = "gids", num_args = 0.., value_delimiter = ',')]
  gids: Vec<usize>,
  /// If a record has an element ID, only output those that contain the
  /// specified IDs. Can be specified more than once, or comma-separated.
  /// If absent, no filter is applied.
  #[arg(short = 'e', long = "eids", num_args = 0.., value_delimiter = ',')]
  eids: Vec<usize>,
  /// If a record has an element type, only output those that contain the
  /// specified types. Can be specified more than once, or comma-separated.
  /// If absent, no filter is applied.
  #[arg(short = 't', long = "etypes", num_args = 0.., value_delimiter = ',')]
  etypes: Vec<ElementType>,
  /// The delimiter used in the CSV.
  #[arg(short = 'd', long = "delim", default_value = ",")]
  delim: char,
  /// Output extra/debug info while parsing and converting.
  #[arg(short = 'v', long = "verbose")]
  verbose: bool,
  /// Path to write output to. If absent, writes to standard output.
  #[arg(short = 'o')]
  output: Option<PathBuf>,
  /// The name of the input F06 file. If -, reads from standard input.
  input: PathBuf,
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
  // parse the file
  let mut f06: F06File = if args.input.as_os_str().eq_ignore_ascii_case("-") {
    OnePassParser::parse_bufread(BufReader::new(io::stdin()))?
  } else if args.input.is_file() {
    if let Some(bn) = args.input.file_name() {
      if let Some(sbn) = bn.to_str() {
        info!("Parsing {}...", sbn);
      }
    } else {
      info!("Parsing...");
    }
    OnePassParser::parse_file(&args.input)?
  } else {
    error!("Provided path either does not exist or is not a file!");
    std::process::exit(1);
  };
  f06.merge_blocks(true);
  f06.merge_potential_headers();
  f06.sort_all_blocks();
  info!("Done parsing.");
  // init the csv writer
  let output: BufWriter<Box<dyn Write>> = BufWriter::new(
    if let Some(ref op) = args.output {
      Box::new(File::create(op)?)
    } else {
      Box::new(io::stdout())
    }
  );
  let mut wtr = csv::Writer::from_writer(output);
  // should we write a record?
  let should_write = |r: &CsvRecord, a: &Cli| -> bool {
    if !a.csv_blocks.is_empty() && a.csv_blocks.contains(&r.block_id) {
      return false;
    }
    if !a.gids.is_empty() && r.gid.is_some_and(|g| a.gids.contains(&g)) {
      return false;
    }
    if !a.eids.is_empty() && r.gid.is_some_and(|e| a.eids.contains(&e)) {
      return false;
    }
    if !a.etypes.is_empty() && r.etype.is_some_and(|e| a.etypes.contains(&e)) {
      return false;
    }
    return true;
  };
  info!("Writing CSV records...");
  // write blocks
  for rec in to_records(&f06, &all_converters()) {
    if should_write(&rec, &args) {
      wtr.write_record(rec.to_fields().map(|f| f.to_string()))?;
    }
  }
  info!("All done.");
  // done
  return Ok(());
}
