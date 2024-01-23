//! This module implements facilities to customise the display of CsvFields.

use std::fmt::Write;

use clap::{Args, ValueEnum};
use f06::util::fmt_f64;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// This enum specifies how floats should be formatted.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Args)]
pub struct FloatFormat {
  /// Specifies a fixed number of decimal places to display numbers with.
  ///
  /// If absent, free-form formatting will be used.
  #[arg(short = 'D', long = "decimals", default_value = "6")]
  pub dec_places: Option<usize>,
  /// Use decimals instead of scientific notation.
  #[arg(short = 'S', long = "no-sci", verbatim_doc_comment)]
  pub no_scientific: bool,
  /// Omit the redundant plus sign for non-negatives.
  #[arg(short = 'P', long = "omit-plus", verbatim_doc_comment)]
  pub no_superfluous_plus: bool,
  /// Use a small 'e' for exponents instead of a capital 'E'.
  #[arg(short = 'E', long = "small-e", verbatim_doc_comment)]
  pub small_e: bool,
}

impl Default for FloatFormat {
  fn default() -> Self {
    return Self {
      dec_places: Some(6),
      no_scientific: false,
      no_superfluous_plus: false,
      small_e: false
    };
  }
}

impl FloatFormat {
  /// Wrties an f64 into a formatter.
  pub fn fmt_f64<W: Write>(&self, f: &mut W, x: f64) -> std::fmt::Result {
    if self.no_scientific {
      return match (self.dec_places, self.no_superfluous_plus) {
        (None, true) => write!(f, "{}", x),
        (None, false) => write!(f, "{:+}", x),
        (Some(d), true) => write!(f, "{:.prec$}", x, prec=d),
        (Some(d), false) => write!(f, "{:+.prec$}", x, prec=d)
      };
    } else if let Some(d) = self.dec_places {
      return fmt_f64(f, x, 0, d, 3, !self.small_e, self.no_superfluous_plus);
    } else {
      return match (self.no_superfluous_plus, self.small_e) {
        (true, true) => write!(f, "{:e}", x),
        (true, false) => write!(f, "{:E}", x),
        (false, true) => write!(f, "{:+e}", x),
        (false, false) => write!(f, "{:+E}", x)
      };
    }
  }
}

/// What to do with blank values?
#[derive(Copy, Clone, Debug, Serialize, Deserialize, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum BlankDisplay {
  /// Prints out a zero.
  Zero,
  /// Prints out a space.
  Space,
  /// Prints a dash.
  Dash,
  /// Prints five dashes.
  Dashes,
  /// Prints nothing (empty field).
  Empty,
}

impl Default for BlankDisplay {
  fn default() -> Self {
    return Self::Zero;
  }
}

impl BlankDisplay {
  /// Returns the string that should be written.
  pub const fn fmt_str(&self) -> &'static str {
    return match self {
      Self::Zero => "0",
      Self::Space => " ",
      Self::Dash => "-",
      Self::Dashes => "-----",
      Self::Empty => ""
    };
  }
}

/// Display/formatting options for CSV fields.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Args)]
pub struct CsvFormatting {
  /// Options for printing out real numbers.
  #[command(flatten)]
  pub reals: FloatFormat,
  /// What to print for blank fields?
  #[arg(short = 'B', long = "blanks", default_value = "zero")]
  pub blanks: BlankDisplay
}

impl Default for CsvFormatting {
  fn default() -> Self {
    return Self { reals: Default::default(), blanks: Default::default() };
  }
}

impl CsvFormatting {
  /// Writes out a CSV field according to this format.
  pub fn fmt<W: Write>(&self, field: &CsvField, f: &mut W) -> std::fmt::Result {
    return match field {
      CsvField::Blank => write!(f, "{}", self.blanks.fmt_str()),
      CsvField::Real(x) => self.reals.fmt_f64(f, *x),
      _ => write!(f, "{}", field)
    }
  }

  /// Turns a CSV field into a string using this formatter.
  pub fn to_string(&self, field: CsvField) -> String {
    return match field {
      CsvField::Blank => self.blanks.fmt_str().to_owned(),
      CsvField::Real(x) => {
        let mut buf = String::new();
        // Bypass format_args!() to avoid write_str with zero-length strs
        self.reals.fmt_f64(&mut buf, x)
          .expect("a Display implementation returned an error unexpectedly");
        buf
      },
      CsvField::String(s) => s,
      _ => field.to_string()
    }
  }
}
