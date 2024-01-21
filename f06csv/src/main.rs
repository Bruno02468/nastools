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
  /// If absent, no grid point ID filter is applied.
  #[arg(short = 'g', long = "gids", num_args = 0.., value_delimiter = ',')]
  gids: Vec<usize>,
  /// If a record has an element ID, only output those that contain the
  /// specified IDs. Can be specified more than once, or comma-separated.
  /// If absent, no element ID filter is applied.
  #[arg(short = 'e', long = "eids", num_args = 0.., value_delimiter = ',')]
  eids: Vec<usize>,
  /// If a record has an element type, only output those that contain the
  /// specified types. Can be specified more than once, or comma-separated.
  /// If absent, no element type filter is applied.
  #[arg(short = 't', long = "etypes", num_args = 0.., value_delimiter = ',')]
  etypes: Vec<ElementType>,
  /// If a record has subcase ID, only output those that contain the
  /// specified IDs. Can be specified more than once, or comma-separated.
  /// If absent, no subcase filter is applied.
  #[arg(short = 's', long = "subcases", num_args = 0.., value_delimiter = ',')]
  subcases: Vec<usize>,
  /// The delimiter used in the CSV.
  #[arg(short = 'd', long = "delim", default_value = ",")]
  delim: char,
  /// Output extra/debug info while parsing and converting.
  #[arg(short = 'v', long = "verbose")]
  verbose: bool,
  /// Enable writing CSV headers. Be warned, they're written every time there's
  /// a change.
  #[arg(short = 'H', long = "headers")]
  headers: bool,
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
  let delim_byte: u8 = args.delim.try_into()
    .expect("Delimiter must not be a special character1");
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(delim_byte)
    .from_writer(output);
  /// Filter only if there is at least one in the filter.
  fn lax_filter<T: PartialEq>(v: &Vec<T>, x: &Option<T>) -> bool {
    return v.is_empty()
      || x.is_none()
      || x.as_ref().is_some_and(|k| v.contains(k));
  }
  // should we write a record?
  let should_write = |r: &CsvRecord, a: &Cli| -> bool {
    let f_gids = lax_filter(&a.gids, &r.gid);
    let f_eids = lax_filter(&a.eids, &r.eid);
    let f_etypes = lax_filter(&a.etypes, &r.etype);
    let f_subcases = lax_filter(&a.subcases, &r.subcase);
    return f_gids && f_eids && f_etypes && f_subcases;
  };
  // write blocks
  info!("Writing CSV records...");
  let mut last_header: Option<&RowHeader> = None;
  for rec in to_records(&f06, &all_converters()) {
    if should_write(&rec, &args) {
      if args.headers {
        let cur_header = &rec.headers;
        let was_none = last_header.is_none();
        last_header = last_header.or(Some(cur_header));
        if last_header != Some(cur_header) || was_none {
          // header change
          last_header = Some(cur_header);
          wtr.write_record(rec.header_as_iter())?;
        }
      }
      wtr.write_record(rec.to_fields().map(|f| f.to_string()))?;
    }
  }
  info!("All done.");
  // done
  return Ok(());
}
