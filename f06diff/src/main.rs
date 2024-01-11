//! This utility loads two F06 files and tells you differences in detected
//! blocks, fields, etcetera. Obviously, you should pass the F06 files
//! generated from running the same model.
//!
//! The main purpose of this program is to compare two solvers running the same
//! model. This way, we can verify MYSTRAN more easily.

#![allow(clippy::needless_return)] // i'll never forgive rust for this

use std::collections::BTreeSet;
use std::io::{self, BufReader};
use std::path::PathBuf;

use clap::Parser;
use log::{LevelFilter, info, error, warn};
use f06::prelude::*;

const INDENT: &str = "  ";

#[derive(Parser)]
#[command(author, version)]
struct Cli {
  /// Output extra/debug info while parsing.
  #[arg(short, long)]
  verbose: bool,
  /// Max number of flags to report individually per block.
  /// Zero prints only a summary, negative prints all flagged positions.
  #[clap(default_value_t = 10)]
  #[arg(short = 'p')]
  print_max_flags: isize,
  /// The settings for the differ.
  #[command(flatten)]
  settings: DiffSettings,
  /// Path to the first file.
  first: PathBuf,
  /// Path to the second file. Set to "-" to read from stdin.
  second: PathBuf,
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
  // check for no ratio and no difference
  let crit = &args.settings.criteria;
  if crit.ratio.is_none() && crit.difference.is_none() {
    warn!("You didn't specify a max difference nor a max ratio.");
    warn!("You'll likely get no useful results, number-wise.");
  }
  // parse the first file
  let mut first = if args.first.is_file() {
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
  let mut second = if args.second.as_os_str().eq_ignore_ascii_case("-") {
    let mut f = OnePassParser::parse_bufread(BufReader::new(io::stdin()))?;
    f.filename = Some("<stdin>".to_string());
    f
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
    b.merge_blocks(true);
    b.merge_potential_headers();
    b.sort_all_blocks();
  }
  // generate the diff
  info!("Generating diff...");
  let diff = F06Diff::compare(&args.settings, &first, &second);
  info!("Done. Report follows.");
  // list not compared blocks
  if !diff.not_compared.is_empty() {
    info!("Blocks that could not be compared:");
  }
  for (br, reason) in diff.not_compared.iter() {
    info!(
      "{}- Subcase {}, {}: {}",
      INDENT,
      br.subcase,
      br.block_type.desc().to_lowercase(),
      reason
    );
  }
  // list compared blocks
  if diff.compared.is_empty() {
    info!("No blocks could be compared.");
  } else if diff.not_compared.is_empty() {
    info!("All blocks were compared:");
  } else {
    info!("Blocks that could be compared:");
  }
  for (br, flags) in diff.compared.iter() {
    info!(
      "{}- Subcase {}, {}:",
      INDENT,
      br.subcase,
      br.block_type.desc().to_lowercase()
    );
    if flags.is_empty() {
      info!("{}{}- No values flagged.", INDENT, INDENT);
    } else {
      // first a summary
      let rows = flags.iter().map(|fp| fp.values.row).collect::<BTreeSet<_>>();
      let cols = flags.iter().map(|fp| fp.values.col).collect::<BTreeSet<_>>();
      info!("{}{}- Flagged {} positions;", INDENT, INDENT, flags.len());
      let count = |s: BTreeSet<NasIndex>, n: &str| {
        if s.len() == 1 {
          info!(
            "{}{}- All in one {}: {};",
            INDENT,
            INDENT,
            n,
            s.first().unwrap()
          );
        } else {
          info!(
            "{}{}- Across {} different {}s;",
            INDENT,
            INDENT,
            s.len(),
            n
          );
        }
      };
      count(rows, "row");
      count(cols, "column");
      // now report specific positions
      let t = match args.print_max_flags.cmp(&0) {
        std::cmp::Ordering::Less => {
          info!(
            "{}{}- Details of all flagged positions:",
            INDENT,
            INDENT
          );
          flags.len()
        },
        std::cmp::Ordering::Equal => {
          info!(
            "{}{}- Details of flagged positions not requested.",
            INDENT,
            INDENT
          );
          continue
        },
        std::cmp::Ordering::Greater => {
          info!(
            "{}{}- Details of flagged positions (limited to {}):",
            INDENT,
            INDENT,
            args.print_max_flags
          );
          args.print_max_flags as usize
        },
      };
      for flag in flags.iter().take(t) {
        info!(
          "{}{}{}- {}, {}:",
          INDENT,
          INDENT,
          INDENT,
          flag.values.row,
          flag.values.col
        );
        info!(
          "{}{}{}{}- Value in {}:\t{}",
          INDENT,
          INDENT,
          INDENT,
          INDENT,
          first.filename.as_deref().unwrap_or("first file"),
          flag.values.val_a
        );
        info!(
          "{}{}{}{}- Value in {}:\t{}",
          INDENT,
          INDENT,
          INDENT,
          INDENT,
          second.filename.as_deref().unwrap_or("second file"),
          flag.values.val_b
        );
        info!(
          "{}{}{}{}- Flag reason: {}.",
          INDENT,
          INDENT,
          INDENT,
          INDENT,
          flag.reason
        );
      }
    }
  }
  return Ok(());
}
