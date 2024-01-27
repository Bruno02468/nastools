//! A command-line application to convert Nastran F06 output to CSV.

#![allow(clippy::needless_return)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use csv::Terminator;
use log::*;
use f06::prelude::*;
use nas_csv::from_f06::templates::all_converters;
use nas_csv::prelude::*;

/// The arguments passed to the converter.
#[derive(Clone, Debug, Parser)]
#[command(
  name = "f06csv",
  author,
  version,
  about = "A command-line utility to convert Nastran F06 output to CSV."
)]
struct Cli {
  /// Path to write output to.
  ///
  /// If absent, writes to standard output.
  #[arg(short = 'o')]
  output: Option<PathBuf>,
  /// CSV blocks to write. Can be specified more than once, or comma-separated.
  ///
  /// You can also write the numerical IDs for shorthand.
  ///
  /// If absent, all blocks are written.
  #[arg(short = 'b', long = "blocks", num_args = 0.., value_delimiter = ',')]
  csv_blocks: Vec<CsvBlockId>,
  /// Grid point ID filter.
  ///
  /// If a record has a grid point ID, only output those that contain the
  /// specified IDs.
  ///
  /// Can be specified more than once, or comma-separated.
  ///
  /// If absent, no grid point ID filter is applied.
  #[arg(short = 'g', long = "gids", num_args = 0.., value_delimiter = ',')]
  gids: Vec<usize>,
  /// Element ID filter.
  ///
  /// If a record has an element ID, only output those that contain the
  /// specified IDs.
  ///
  /// Can be specified more than once, or comma-separated.
  ///
  /// If absent, no element ID filter is applied.
  #[arg(short = 'e', long = "eids", num_args = 0.., value_delimiter = ',')]
  eids: Vec<usize>,
  /// Element type filter.
  ///
  /// If a record has an element type, only output those that contain the
  /// specified types.
  ///
  /// Can be specified more than once, or comma-separated.
  ///
  /// If absent, no element type filter is applied.
  #[arg(short = 't', long = "etypes", num_args = 0.., value_delimiter = ',')]
  etypes: Vec<ElementType>,
  /// Subcase filter.
  ///
  /// If a record has subcase ID, only output those that contain the
  /// specified IDs.
  ///
  /// Can be specified more than once, or comma-separated.
  ///
  /// If absent, no subcase filter is applied.
  #[arg(short = 's', long = "subcases", num_args = 0.., value_delimiter = ',')]
  subcases: Vec<usize>,
  /// Enable writing CSV headers.
  ///
  /// Be warned, they're written every time there's a change.
  #[arg(short = 'H', long = "headers")]
  headers: bool,
  /// The delimiter used in the CSV.
  #[arg(short = 'd', long, default_value = ",", verbatim_doc_comment)]
  delim: char,
  /// Use CRLF (Windows) line breaks. Default is Unix (LF).
  #[arg(long = "crlf", verbatim_doc_comment)]
  crlf: bool,
  /// Formatting options.
  #[command(flatten)]
  fmtr: CsvFormatting,
  /// Output extra/debug info while parsing and converting.
  #[arg(short = 'v', long = "verbose", verbatim_doc_comment)]
  verbose: bool,
  /// The name of the input F06 file.
  ///
  /// If -, reads from standard input.
  input: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
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
  let term = if args.crlf { Terminator::CRLF } else { Terminator::default() };
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(delim_byte)
    .terminator(term)
    .from_writer(output);
  /// Filter only if there is at least one in the filter.
  fn lax_filter<T: PartialEq>(v: &Vec<T>, x: &Option<T>) -> bool {
    return v.is_empty()
      || x.is_none()
      || x.as_ref().is_some_and(|k| v.contains(k));
  }
  // should we write a record?
  let should_write = |r: &CsvRecord, a: &Cli| -> bool {
    let f_blocks = lax_filter(&a.csv_blocks, &Some(r.block_id));
    let f_gids = lax_filter(&a.gids, &r.gid);
    let f_eids = lax_filter(&a.eids, &r.eid);
    let f_etypes = lax_filter(&a.etypes, &r.etype);
    let f_subcases = lax_filter(&a.subcases, &r.subcase);
    return f_gids && f_eids && f_etypes && f_subcases && f_blocks;
  };
  // determine padding
  let largest: Option<usize> = if args.fmtr.align != Alignment::None {
    to_records(&f06, &all_converters())
      .filter_map(|rec| {
        if should_write(&rec, &args) && rec.block_id != CsvBlockId::Metadata {
          let h = if args.headers {
            rec.header_as_iter().map(|f| f.len()).max()
          } else {
            None
          };
          let n = rec.to_fields().map(|f| f.to_string().len()).max();
          return n.max(h);
        } else {
          return None;
        }
      }).max()
  } else {
    None
  };
  // padding fn
  let pad = |s: &str| -> String {
    if let Some(w) = largest {
      if s.len() > w {
        return s.to_owned();
      }
      let p1 = w - s.len();
      let ps = p1/2;
      let pb = p1 - ps;
      let (lpad, rpad) = match args.fmtr.align {
        Alignment::None => return s.to_owned(),
        Alignment::Right => (p1, 0),
        Alignment::Left => (0, p1),
        Alignment::Center => (pb, ps),
      };
      return format!(
        "{}{}{}",
        " ".repeat(lpad),
        s,
        " ".repeat(rpad),
      );
    } else {
      return s.to_owned();
    }
  };
  // write blocks
  info!("Writing CSV records...");
  let mut last_header: Option<(&RowHeader, CsvBlockId)> = None;
  for rec in to_records(&f06, &all_converters()) {
    if should_write(&rec, &args) {
      if args.headers {
        let cur_header = &rec.headers;
        let cur_bid = rec.block_id;
        let was_none = last_header.is_none();
        last_header = last_header.or(Some((cur_header, cur_bid)));
        if last_header != Some((cur_header, cur_bid)) || was_none {
          // header change
          last_header = Some((cur_header, cur_bid));
          wtr.write_record(rec.header_as_iter().map(pad))?;
        }
      }
      wtr.write_record(rec.to_fields().map(|f| pad(&f.to_string())))?;
    }
  }
  info!("All done.");
  // done
  return Ok(());
}
