//! This module implements an App, which is a basic framework around which one
//! can construct interaction with `nastester`, be it automated (e.g. a CLI) or
//! fully-interactive (like the GUI).

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use f06::prelude::*;
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

  /// Returns the names of solvers, in order.
  pub(crate) fn solvers_names(&self) -> impl Iterator<Item = (&str, Uuid)> {
    let ordering: BTreeMap<&str, Uuid> = self.solvers.iter()
      .map(|(u, d)| (d.nickname.as_str(), *u))
      .collect();
    return ordering.into_iter();
  }

  /// Iterates over solvers by name.
  pub(crate) fn solvers_by_name(
    &self
  ) -> impl Iterator<Item = (Uuid, &RunnableSolver)> {
    return self.solvers_names().map(|(_, u)| (
      u,
      self.solvers.get(&u).expect("invalid solver UUID")
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

  /// Generates a job for a deck and a solver pick.
  pub(crate) fn gen_job(
    &mut self,
    deck_uuid: Uuid,
    pick: SolverPick
  ) -> Option<Job> {
    if let Some(solver) = self.get_solver(pick).cloned() {
      if let Some((deck, res)) = self.get_deck(deck_uuid) {
        return Some(Job {
          deck: deck.clone(),
          pick,
          target: res,
          solver: solver.clone(),
          crit_sets: self.suite.criteria_sets.clone()
        });
      }
    }
    return None;
  }

  /// Enqueues a run for a single deck. Does nothing if there isn't a solver
  /// picked yet. This might lock, use enqueue_deck safe if in doubt.
  pub(crate) fn enqueue_deck(&mut self, deck_uuid: Uuid, pick: SolverPick) {
    if let Some(job) = self.gen_job(deck_uuid, pick) {
      self.runner.job_queue.lock().expect("mutex poisoned").push_back(job);
      self.set_run_state(deck_uuid, pick, RunState::Enqueued);
    }
  }

  /// Enqueues a deck in a separate thread to prevent UI locking.
  pub(crate) fn enqueue_deck_safe(&mut self, deck: Uuid, pick: SolverPick) {
    let queue = self.runner.job_queue.clone();
    let state = self.get_run_state(deck);
    if let Some(job) = self.gen_job(deck, pick) {
      thread::spawn(move || {
        queue.lock().unwrap().push_back(job);
        *state.lock().unwrap().get_mut(pick) = RunState::Enqueued;
      });
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
      results.recompute_flagged(deck, &critsets);
    }
  }

  /// Re-computes all flagged values in the UI thread.
  pub(crate) fn recompute_all_flagged(&mut self) {
    let uuids = self.suite.decks.keys().copied().collect::<Vec<_>>();
    for deck in uuids {
      self.recompute_flagged(deck);
    }
  }
}
