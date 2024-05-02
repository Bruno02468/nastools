//! This submodule defines structures for testing results.

use std::collections::{BTreeMap, BTreeSet};

use f06::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::running::SolverPick;
use crate::suite::*;

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

/// This structure holds extraction data in matrix form.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExtractionMatrix {
  /// The solver pick this is for.
  pub(crate) solver: SolverPick,
  /// The extraction number this is for.
  pub(crate) extraction_num: usize,
  /// The resulting sub-blocks.
  pub(crate) blocks: Vec<FinalBlock>
}


/// These are the results for a single deck.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DeckResults {
  /// The F06 file from the reference solver.
  pub(crate) ref_f06: RunState,
  /// The F06 file from the solver under test.
  pub(crate) test_f06: RunState,
  /// Contains the flagged values.
  pub(crate) flagged: Vec<Option<BTreeSet<DatumIndex>>>
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

  /// Recomputes the flagged values.
  pub(crate) fn recompute_flagged(
    &mut self,
    deck: &Deck,
    crit_sets: &BTreeMap<Uuid, NamedCriteria>
  ) {
    self.flagged.clear();
    let pair = (&self.ref_f06, &self.test_f06);
    if let (RunState::Finished(r), RunState::Finished(t)) = pair {
      for (exn, crit_uuid) in deck.extractions.iter() {
        if let Some(critset) = crit_uuid.and_then(|u| crit_sets.get(&u)) {
          let in_ref = exn.lookup(r).collect::<BTreeSet<_>>();
          let in_test = exn.lookup(t).collect::<BTreeSet<_>>();
          let in_either = in_ref.union(&in_test).collect::<BTreeSet<_>>();
          let dxn = in_ref.symmetric_difference(&in_test)
            .collect::<BTreeSet<_>>();
          let mut flagged: BTreeSet<DatumIndex> = BTreeSet::new();
          if exn.dxn == DisjunctionBehaviour::Flag {
            flagged.extend(dxn);
          }
          let get = |f: &F06File, ix: &DatumIndex| -> Option<F06Number> {
            let v = ix.get_from(f);
            if v.is_err() && exn.dxn == DisjunctionBehaviour::AssumeZeroes {
              return Some(0.0.into());
            } else {
              return Some(v.unwrap());
            }
          };
          for ix in in_either {
            let val_ref = get(r, ix);
            let val_test = get(t, ix);
            if let (Some(rv), Some(tv)) = (val_ref, val_test) {
              if critset.criteria.check(rv.into(), tv.into()).is_some() {
                flagged.insert(*ix);
              }
            }
          }
          self.flagged.push(Some(flagged));
        } else {
          self.flagged.push(None);
        }
      }
    }
  }
}
