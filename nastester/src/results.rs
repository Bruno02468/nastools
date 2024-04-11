//! This submodule defines structures for testing results.

use f06::prelude::*;
use serde::{Deserialize, Serialize};

/// These are the results for a single deck.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DeckResults {
  /// The F06 file from the reference solver.
  pub(crate) ref_f06: Option<F06File>,
  /// The F06 file from the solver under test.
  pub(crate) test_f06: Option<F06File>

}
