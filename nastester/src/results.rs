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

/// This structure holds extraction results: blocks and flagged indexes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExtractionResults {
  /// The extraction number this is for.
  pub(crate) extraction_num: usize,
  /// The resulting sub-blocks gotten from the reference solver.
  pub(crate) blocks_ref: Vec<FinalBlock>,
  /// The resulting sub-blocks gotten from the solver under test.
  pub(crate) blocks_test: Vec<FinalBlock>,
  /// The flagged indices.
  pub(crate) flagged: Option<BTreeSet<DatumIndex>>,
  /// The extracted indices.
  pub(crate) extracted: BTreeSet<DatumIndex>
}


/// These are the results for a single deck.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DeckResults {
  /// The F06 file from the reference solver.
  pub(crate) ref_f06: RunState,
  /// The F06 file from the solver under test.
  pub(crate) test_f06: RunState,
  /// Contains the results for each extraction.
  pub(crate) extractions: Vec<ExtractionResults>,
  /// Contains all flagged indices.
  pub(crate) flagged: BTreeSet<DatumIndex>,
  /// Contains all extracted indices.
  pub(crate) extracted: BTreeSet<DatumIndex>
}

impl DeckResults {
  /// Gets a reference to a run state.
  pub(crate) fn get(&self, solver: SolverPick) -> &RunState {
    return match solver {
      SolverPick::Reference => &self.ref_f06,
      SolverPick::Testing => &self.test_f06,
    };
  }

  /// Gets a mutable reference to a run state.
  pub(crate) fn get_mut(&mut self, solver: SolverPick) -> &mut RunState {
    return match solver {
      SolverPick::Reference => &mut self.ref_f06,
      SolverPick::Testing => &mut self.test_f06,
    };
  }

  /// Clears all flagged values.
  pub(crate) fn clear_flags(&mut self) {
    for res in self.extractions.iter_mut() {
      res.flagged = None;
    }
    self.flagged.clear();
  }

  /// Clears a run's results.
  pub(crate) fn clear_of(&mut self, pick: SolverPick) {
    *self.get_mut(pick) = RunState::Ready;
    for res in self.extractions.iter_mut() {
      res.flagged = None;
      match pick {
        SolverPick::Reference => &mut res.blocks_ref,
        SolverPick::Testing => &mut res.blocks_test,
      }.clear();
    }
  }

  /// Recomputes the extraction results (sub-blocks and flagged values).
  pub(crate) fn recompute_extractions(
    &mut self,
    deck: &Deck,
    crit_sets: &BTreeMap<Uuid, NamedCriteria>
  ) {
    self.extractions.clear();
    let pair = (&self.ref_f06, &self.test_f06);
    if let (RunState::Finished(r), RunState::Finished(t)) = pair {
      for (i, (exn, crit_uuid)) in deck.extractions.iter().enumerate() {
        let mut res = ExtractionResults {
          extraction_num: i,
          blocks_ref: exn.blockify(r),
          blocks_test: exn.blockify(t),
          flagged: None,
          extracted: BTreeSet::new()
        };
        res.extracted.extend(exn.lookup(r));
        res.extracted.extend(exn.lookup(t));
        if let Some(critset) = crit_uuid.and_then(|u| crit_sets.get(&u)) {
          let in_ref = exn.lookup(r).collect::<BTreeSet<_>>();
          let in_test = exn.lookup(t).collect::<BTreeSet<_>>();
          let in_either = in_ref
          .union(&in_test)
            .copied()
            .collect::<BTreeSet<_>>();
          let dxn = in_ref
            .symmetric_difference(&in_test)
            .copied()
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
            let val_ref = get(r, &ix);
            let val_test = get(t, &ix);
            if let (Some(rv), Some(tv)) = (val_ref, val_test) {
              if critset.criteria.check(rv.into(), tv.into()).is_some() {
                flagged.insert(ix);
              }
            }
          }
          self.flagged.extend(flagged.iter().copied());
          res.flagged = Some(flagged);
        }
        self.extracted.extend(res.extracted.iter().copied());
        self.extractions.push(res);
      }
    }
  }

  /// Returns all block refs in the results set.
  pub(crate) fn all_block_refs(&self) -> Vec<BlockRef> {
    let mut v: Vec<BlockRef> = Vec::new();
    for pick in SolverPick::all() {
      if let RunState::Finished(f) = self.get(*pick) {
        v.extend(f.all_blocks(true).map(|b| b.block_ref()));
      }
    }
    v.sort();
    v.dedup();
    return v;
  }

  /// Returns the total number of flagged values.
  pub(crate) fn num_flagged(&self) -> usize {
    return self.extractions.iter()
      .filter_map(|er| er.flagged.as_ref().map(|v| v.len()))
      .sum();
  }
}
