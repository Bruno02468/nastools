//! This submodule defines structures for testing results.

use f06::prelude::*;
use serde::{Deserialize, Serialize};

use crate::running::SolverPick;

/// Enum containing a deck status.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) enum RunState {
  /// Not yet run.
  #[default]
  Ready,
  /// Enqueued.
  Enqueued,
  /// Running
  Running,
  /// Finished, F06 file present.
  Finished(F06File),
  /// Run failed, contains error.
  Error(String)
}

impl<T: ToString> From<Result<F06File, T>> for RunState {
  fn from(value: Result<F06File, T>) -> Self {
    return match value {
      Ok(f) => Self::Finished(f),
      Err(e) => Self::Error(e.to_string()),
    }
  }
}

/// These are the results for a single deck.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DeckResults {
  /// The F06 file from the reference solver.
  pub(crate) ref_f06: RunState,
  /// The F06 file from the solver under test.
  pub(crate) test_f06: RunState
}

impl DeckResults {
  /// Gets a reference to a result.
  pub(crate) fn get(&self, solver: SolverPick) -> &RunState {
    return match solver {
      SolverPick::Reference => &self.ref_f06,
      SolverPick::Testing => &self.test_f06,
    };
  }

  /// Gets a mutable reference to a result.
  pub(crate) fn get_mut(&mut self, solver: SolverPick) -> &mut RunState {
    return match solver {
      SolverPick::Reference => &mut self.ref_f06,
      SolverPick::Testing => &mut self.test_f06,
    };
  }
}
