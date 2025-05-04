//! This simple submodule implements cross-F06 comparison.

use serde::{Deserialize, Serialize};

use crate::utils::OneOrMany;

/// A comparison takes two or more F06 files
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Comparison {
  /// The name of this comparison.
  pub(crate) name: String,
  /// The name of the reference F06 file.
  pub(crate) reference_f06: String,
  /// The name of the test F06 file.
  pub(crate) test_f06: String,
  /// Data extractions to pull.
  #[serde(alias = "extraction")]
  pub(crate) extractions: OneOrMany<String>,
  /// Comparison criteria to apply.
  #[serde(alias = "criterion")]
  pub(crate) criteria: String,
  /// Output a report to a file.
  #[serde(default)]
  pub(crate) report: Option<String>,
}
