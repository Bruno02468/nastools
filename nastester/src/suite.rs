//! This module defines a "test suite": a series of input files and comparison
//! criteria.

use std::collections::BTreeMap;
use std::path::PathBuf;

use f06::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Extension for suite files.
pub(crate) const SUITE_FILE_EXTENSION: &str = "nts";

/// Extensions for bulk data files.
pub(crate) const DECK_EXTENSIONS: &[&str] =
  &["bdf", "nas", "dat", "BDF", "NAS", "DAT"];

/// This is an input file, a.k.a. a "deck", along with pairs of extractions
/// and criteria-set IDs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Deck {
  /// Path to the input file. It'll only be read, don't worry.
  pub(crate) in_file: PathBuf,
  /// A list of extraction and criteria-ID pairs.
  pub(crate) extractions: Vec<(Extraction, Option<Uuid>)>,
}

impl Deck {
  /// Returns a display name for the deck, based on its base file name.
  pub(crate) fn name(&self) -> &str {
    return self
      .in_file
      .file_name()
      .and_then(|s| s.to_str())
      .unwrap_or("<non-displayable filename>");
  }
}

/// This is a named criteria set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct NamedCriteria {
  /// The name for this criteria set.
  pub(crate) name: String,
  /// The actual number comparison criteria.
  pub(crate) criteria: Criteria,
}

/// This is a test suite. It contains decks and criteria sets.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Suite {
  /// The test decks to run.
  pub(crate) decks: BTreeMap<Uuid, Deck>,
  /// The named criteria sets.
  pub(crate) criteria_sets: BTreeMap<Uuid, NamedCriteria>,
}
