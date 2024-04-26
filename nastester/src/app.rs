//! This module implements an App, which is a basic framework around which one
//! can construct interaction with `nastester`, be it automated (e.g. a CLI) or
//! fully-interactive (like the GUI).

use std::collections::BTreeMap;
use std::path::PathBuf;

use f06::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::results::DeckResults;
use crate::running::*;
use crate::suite::*;

/// This contains everything the app should be doing right now.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct AppState {
  /// The solvers currently known to the app.
  pub(crate) solvers: BTreeMap<Uuid, RunnableSolver>,
  /// The current test suite.
  pub(crate) suite: Suite,
  /// The currently-selected reference solver, if any.
  pub(crate) ref_solver: Option<Uuid>,
  /// The currently-selected solver under test, if any.
  pub(crate) test_solver: Option<Uuid>,
  /// The results currently loaded for the decks.
  pub(crate) results: BTreeMap<Uuid, DeckResults>
}

impl AppState {
  /// Adds a deck file to the app's current suite.
  pub(crate) fn add_deck(&mut self, in_file: PathBuf) {
    let deck = Deck {
      in_file,
      extractions: Vec::new(),
    };
    let uuid = Uuid::new_v4();
    self.suite.decks.insert(uuid, deck);
  }

  /// Adds a solver from a known binary.
  pub(crate) fn add_solver_bin(&mut self, binary: PathBuf) {
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
  }

  /// Adds a solver from an F06 directory.
  pub(crate) fn add_solver_dir(&mut self, dir: PathBuf) {
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
  ) -> impl Iterator<Item = (Uuid, &Deck, Option<&DeckResults>)> {
    return self.decks_names().map(|(_, u)| (
      u,
      self.suite.decks.get(&u).expect("invalid deck UUID"),
      self.results.get(&u))
    )
  }

  /// Returns a deck and its results.
  pub(crate) fn get_deck(&self, uuid: Uuid) -> Option<(&Deck, Option<&DeckResults>)> {
    if let Some(deck) = self.suite.decks.get(&uuid) {
      return Some((
        deck,
        self.results.get(&uuid)
      ));
    } else {
      return None;
    }
  }
}
