//! This module implements an App, which is a basic framework around which one
//! can construct interaction with `nastester`, be it automated (e.g. a CLI) or
//! fully-interactive (like the GUI).

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use f06::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::results::DeckResults;
use crate::results::RunState;
use crate::running::*;
use crate::suite::*;

/// This contains everything the app should be doing right now.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct AppState {
  /// The solvers currently known to the app.
  pub(crate) solvers: BTreeMap<Uuid, RunnableSolver>,
  /// The current test suite.
  pub(crate) suite: Suite,
  /// The runner.
  pub(crate) runner: Runner
}

impl AppState {
  /// Adds a deck file to the app's current suite.
  pub(crate) fn add_deck(&mut self, in_file: PathBuf) -> Uuid {
    let deck = Deck {
      in_file,
      extractions: Vec::new(),
    };
    let uuid = Uuid::new_v4();
    self.suite.decks.insert(uuid, deck);
    return uuid;
  }

  /// Adds a solver from a known binary.
  pub(crate) fn add_solver_bin(&mut self, binary: PathBuf) -> Uuid {
    let nickname = binary
      .file_name()
      .and_then(|s| s.to_str())
      .unwrap_or("<unnamed>")
      .to_string();
    let solver = RunnableSolver {
      kind: Solver::Mystran,
      nickname,
      method: RunMethod::RunSolver(binary)
    };
    let uuid = Uuid::new_v4();
    self.solvers.insert(uuid, solver);
    return uuid;
  }

  /// Adds a solver from an F06 directory.
  pub(crate) fn add_solver_dir(&mut self, dir: PathBuf) -> Uuid {
    let nickname = dir
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("<unnamed>")
    .to_string();
    let solver = RunnableSolver {
      kind: Solver::Simcenter,
      nickname,
      method: RunMethod::ImportFromDir(dir)
    };
    let uuid = Uuid::new_v4();
    self.solvers.insert(uuid, solver);
    return uuid;
  }

  /// Adds a new criteria set.
  pub(crate) fn add_crit_set(&mut self) -> Uuid {
    let uuid = Uuid::new_v4();
    let critset = NamedCriteria {
      name: format!("critset_{}", self.suite.criteria_sets.len() + 1),
      criteria: Criteria::default()
    };
    self.suite.criteria_sets.insert(uuid, critset);
    return uuid;
  }

  /// Returns decks in order of name with their UUIDs.
  pub(crate) fn decks_names(&self) -> impl Iterator<Item = (&str, Uuid)> {
    let ordering: BTreeMap<&str, Uuid> = self.suite.decks.iter()
      .map(|(u, d)| (d.name(), *u))
      .collect();
    return ordering.into_iter();
  }

  /// Iterates over decks and their results, sorted by name.
  pub(crate) fn decks_by_name(
    &self
  ) -> impl Iterator<Item = (Uuid, &Deck, Option<Arc<Mutex<DeckResults>>>)> {
    return self.decks_names().map(|(_, u)| (
      u,
      self.suite.decks.get(&u).expect("invalid deck UUID"),
      self.runner.results.get(&u).cloned()
    ))
  }

  /// Returns a deck and its results.
  pub(crate) fn get_deck(
    &mut self,
    uuid: Uuid
  ) -> Option<(&Deck, Arc<Mutex<DeckResults>>)> {
    if let Some(deck) = self.suite.decks.get(&uuid) {
      return Some((
        deck,
        self.runner.results.entry(uuid).or_default().clone()
      ));
    } else {
      return None;
    }
  }

  /// Returns a mutable reference into a deck and its results.
  pub(crate) fn get_deck_mut(
    &mut self,
    uuid: Uuid
  ) -> Option<(&mut Deck, Option<Arc<Mutex<DeckResults>>>)> {
    if let Some(deck) = self.suite.decks.get_mut(&uuid) {
      return Some((
        deck,
        self.runner.results.get(&uuid).cloned()
      ));
    } else {
      return None;
    }
  }

  /// Deletes a criteria set and removes it from decks.
  pub(crate) fn delete_crit_set(&mut self, uuid: Uuid) {
    self.suite.criteria_sets.remove(&uuid);
    self.suite.decks.values_mut()
      .for_each(|d| d.extractions.iter_mut().for_each(
        |(_, u)| if u == &Some(uuid) { *u = None }
      ))
  }

  /// Clears all results.
  pub(crate) fn clear_results(&mut self) {
    self.runner.results.clear();
  }

  /// Gets a handle into a run state.
  pub(crate) fn get_run_state(
    &mut self,
    deck: Uuid
  ) -> Arc<Mutex<DeckResults>> {
    let tgt = self.runner.results
      .entry(deck)
      .or_insert(Arc::new(Mutex::new(DeckResults::default())));
    return tgt.clone();
  }

  /// Sets a run state. Might block!
  pub(crate) fn set_run_state(
    &mut self,
    deck: Uuid,
    pick: SolverPick,
    state: RunState
  ) {
    let handle = self.get_run_state(deck);
    *handle.lock().expect("mutex poisoned").get_mut(pick) = state;
  }

  /// Gets the current picked solver for something.
  pub(crate) fn get_solver(&self, pick: SolverPick) -> Option<&RunnableSolver> {
    return self.runner.get_solver(pick).and_then(|u| self.solvers.get(&u));
  }

  /// Enqueues a run for a single deck. Does nothing if there isn't a solver
  /// picked yet.
  pub(crate) fn enqueue_deck(&mut self, deck_uuid: Uuid, pick: SolverPick) {
    if let Some(solver) = self.get_solver(pick).cloned() {
      if let Some((deck, res)) = self.get_deck(deck_uuid) {
        let job = Job {
          deck: deck.clone(),
          pick,
          target: res,
          solver: solver.clone(),
        };
        self.runner.job_queue.lock().expect("mutex poisoned").push_back(job);
        self.set_run_state(deck_uuid, pick, RunState::Enqueued);
      }
    }

  }

  /// Enqueues all jobs for a solver pick.
  pub(crate) fn enqueue_solver(&mut self, pick: SolverPick) {
    let decks = self.suite.decks.keys().copied().collect::<Vec<_>>();
    for u in decks {
      self.enqueue_deck(u, pick);
    }
  }

  /// Enqueues all jobs for all solvers.
  pub(crate) fn enqueue_all(&mut self) {
    self.enqueue_solver(SolverPick::Reference);
    self.enqueue_solver(SolverPick::Testing);
  }

  /// Clears the job queue.
  pub(crate) fn clear_queue(&self) {
    self.runner.job_queue.lock().unwrap().clear();
  }

  /// Spawns threads to run the queue.
  pub(crate) fn run_queue(&self) {
    let relaxed = std::sync::atomic::Ordering::Relaxed;
    let runner = |queue: Arc<Mutex<VecDeque<Job>>>, mj: Arc<AtomicUsize>| {
      let relaxed = std::sync::atomic::Ordering::Relaxed;
      mj.fetch_add(1, relaxed);
      log::debug!("Runner {} spawned!", mj.load(relaxed));
      loop {
        let job_opt = queue.lock().expect("lock poisoned").pop_front();
        if let Some(job) = job_opt {
          job.run();
        } else {
          break;
        }
      }
      log::debug!("Runner {} done!", mj.load(relaxed));
      mj.fetch_sub(1, relaxed);
    };
    let nt = if self.runner.max_jobs == 0 {
      num_cpus::get()
    } else {
      self.runner.max_jobs
    };
    for jn in 0..nt {
      if self.runner.current_jobs.load(relaxed) < nt {
        let queue = self.runner.job_queue.clone();
        let job_count = self.runner.current_jobs.clone();
        thread::Builder::new()
          .name(format!("job_runner_{}", jn+1))
          .spawn(move || runner(queue, job_count))
          .expect("failed to spawn runner thread");
      }
    }
  }

  /// Re-computes the flagged values for a deck.
  pub(crate) fn recompute_flagged(&mut self, deck: Uuid) {
    let critsets = self.suite.criteria_sets.clone();
    if let Some((deck, results_mtx)) = self.get_deck(deck) {
      let mut results = results_mtx.lock().expect("mutex poisoned");
      results.flagged.clear();
      let pair = (&results.ref_f06, &results.test_f06);
      if let (RunState::Finished(r), RunState::Finished(t)) = pair {
        for (exn, crit_uuid) in deck.extractions.iter() {
          if let Some(critset) = crit_uuid.and_then(|u| critsets.get(&u)) {
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
            results_mtx.lock().expect("poisoned").flagged.push(Some(flagged));
          } else {
            results_mtx.lock().expect("mutex poisoned").flagged.push(None);
          }
        }
      }
    }
  }

  /// Re-computes all flagged values in a background thread.
  pub(crate) fn recompute_all_flagged(&mut self) {
    let decks = self.suite.decks.keys().cloned().collect::<Vec<_>>();
    let mref = Arc::new(Mutex::new(self));
    decks.par_iter()
      .map(|u| (u, mref.clone()))
      .for_each(|(u, s)| {
        if let Ok(mut s) = s.lock() { s.recompute_flagged(*u); }
      })
  }
}
