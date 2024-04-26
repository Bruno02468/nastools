//! This module implements the top-level GUI for `nastester`.

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;

use egui::{Align, Color32, ComboBox, Id, Layout, Rect, TextStyle, Ui, Visuals, WidgetText};
use egui_extras::{Column, TableBuilder};
use f06::f06file::extraction::{Extraction, Specifier, SpecifierType};
use log::info;
use native_dialog::{MessageDialog, MessageType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::*;
use crate::running::*;
use crate::suite::*;

/// This enum contains the different views that can be rendered.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) enum View {
  /// Default view: the decks.
  #[default]
  Decks,
  /// The solvers.
  Solvers,
  /// The criteria sets.
  CriteriaSets,
  /// The logs.
  Logs,
  /// A specific deck.
  Deck(Uuid),
  /// A specific solver.
  Solver(Uuid),
  /// A specific criteria set.
  CriteriaSet(Uuid)
}

/// This struct rerpresents the GUI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Gui {
  /// The inner app state.
  pub(crate) state: AppState,
  /// The current view.
  pub(crate) view: View,
  /// Whether we have a save file for the test suite.
  pub(crate) suite_file: Option<PathBuf>,
  /// Whether the current suite has been saved.
  pub(crate) suite_clean: bool,
  /// Text fields that are not 1:1 with state data, so they need to stay
  /// "invalid" sometimes. These are cleared when the view changes.
  pub(crate) text_fields: HashMap<Id, String>
}

impl Default for Gui {
  fn default() -> Self {
    return Self {
      state: AppState::default(),
      view: View::Decks,
      suite_file: None,
      suite_clean: true,
      text_fields: HashMap::new()
    };
  }
}

/// Fallible function for GUI inner stuff.
type GuiFn<T> = fn(&mut Gui, &mut Ui) -> Result<T, Box<dyn Error>>;

impl Gui {
  /// Constructs a new Gui with an eframe creation context.
  pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    Self::default()
  }

  /// Save the suite. Returns whether the save happened.
  fn save_suite(&mut self, _ui: &mut Ui) -> Result<bool, Box<dyn Error>> {
    if self.suite_file.is_none() {
      // show dialog
      let picked = rfd::FileDialog::new()
        .add_filter("nastester suite file", &[SUITE_FILE_EXTENSION])
        .set_title("Save suite to file...")
        .set_can_create_directories(true)
        .save_file();
      if let Some(mut p) = picked {
        if p.extension().is_none() {
          p.set_extension(SUITE_FILE_EXTENSION);
        }
        self.suite_file = Some(p);
      }
    }
    if let Some(ref p) = self.suite_file {
      let file = File::create(p)?;
      let mut writer = BufWriter::new(file);
      serde_json::to_writer_pretty(&mut writer, &self.state.suite)?;
      writer.flush()?;
      self.suite_clean = true;
      log::info!("Saved suite to {}.", p.display());
      return Ok(true);
    } else {
      log::info!("Suite saving cancelled or no file chosen.");
      return Ok(false)
    }
  }

  /// Ensure the suite is clean before changing paths of closing. Returns
  /// whether the cleanliness was ensured.
  fn sure_clean(&mut self, ui: &mut Ui) -> Result<bool, Box<dyn Error>> {
    if !self.suite_clean {
      let wants_save = MessageDialog::new()
        .set_title("Are you sure?")
        .set_text("You have unsaved changes. Do you want to save them before?")
        .show_confirm()
        .unwrap_or(true);
      if wants_save {
        self.save_suite(ui)?;
      }
    }
    return Ok(self.suite_clean);
  }

  /// "Save as" -- clear the save path and save, basically.
  fn save_suite_as(&mut self, ui: &mut Ui) -> Result<bool, Box<dyn Error>> {
    self.sure_clean(ui)?;
    self.suite_clean = false;
    self.suite_file = None;
    return self.save_suite(ui);
  }

  /// Load a suite. Returns whether the load happened.
  fn load_suite(&mut self, ui: &mut Ui) -> Result<bool, Box<dyn Error>> {
    self.sure_clean(ui)?;
    self.suite_file = rfd::FileDialog::new()
      .add_filter("nastester suite file", &[SUITE_FILE_EXTENSION])
      .add_filter("All files", &["*"])
      .set_title("Load suite from file...")
      .set_can_create_directories(true)
      .pick_file();
    if let Some(ref p) = self.suite_file {
      let file = File::open(p)?;
      let reader = BufReader::new(file);
      self.state.suite = serde_json::from_reader(reader)?;
      log::info!("Loaded suite from {}.", p.display());
      return Ok(true);
    }
    log::info!("Suite loading cancelled or no file chosen.");
    return Ok(false);
  }

  /// Add one or more decks. Returns how many.
  fn add_decks(&mut self, _ui: &mut Ui) -> Result<usize, Box<dyn Error>> {
    let deck_files = rfd::FileDialog::new()
      .add_filter("NASTRAN input files", DECK_EXTENSIONS)
      .add_filter("All files", &["*"])
      .set_title("Choose input files...")
      .set_can_create_directories(true)
      .pick_files();
    if let Some(v) = deck_files {
      let mut total = 0;
      for in_file in v {
        if in_file.is_file() {
          log::info!("Added deck from file {}.", in_file.display());
          self.suite_clean = false;
          self.state.add_deck(in_file);
          total += 1;
        } else {
          log::info!("Tried to add deck from non-file {}!", in_file.display());
        }
      }
      return Ok(total);
    } else {
      log::info!("Deck addition cancelled by user or no file(s) selected.");
      return Ok(0);
    }
  }

  /// Add a solver binary. Returns whether it's been added.
  fn add_solver_bin(&mut self, _ui: &mut Ui) -> Result<bool, Box<dyn Error>> {
    let mut dialog = rfd::FileDialog::new()
      .set_title("Choose solver binary...")
      .set_can_create_directories(true);
    if !BINARY_EXTENSIONS.is_empty() {
      dialog = dialog.add_filter("Executable file", BINARY_EXTENSIONS);
    }
    let binary = dialog.pick_file();
    if let Some(bin) = binary {
      log::info!("Added deck from file {}.", bin.display());
      self.state.add_solver_bin(bin);
      return Ok(true);
    } else {
      log::info!("Solver addition cancelled by user or no binary selected.");
      return Ok(false);
    }
  }

  /// Add a solver as an F06 directory. Returns whether it's been added.
  fn add_solver_dir(&mut self, _ui: &mut Ui) -> Result<bool, Box<dyn Error>> {
    let directory = rfd::FileDialog::new()
      .set_title("Choose F06 directory...")
      .set_can_create_directories(true)
      .pick_folder();
    if let Some(dir) = directory {
      log::info!("Added solver as F06 directory {}.", dir.display());
      self.state.add_solver_dir(dir);
      return Ok(true);
    } else {
      log::info!("Solver addition cancelled by user or no folder selected.");
      return Ok(false);
    }
  }

  /// Run a function and, if an error happens, do a pop-up.
  fn show_error(err: Box<dyn Error>) {
    MessageDialog::new()
      .set_title("Error")
      .set_type(MessageType::Error)
      .set_text(err.to_string().as_str())
      .show_alert()
      .ok();
  }

  /// Try to run an inner subroutine and, if it fails, show a pop-up.
  fn try_run<T>(&mut self, ui: &mut Ui, f: GuiFn<T>) {
    if let Err(err) = f(self, ui) {
      Self::show_error(err);
    }
  }

  /// Returns an editable text field buffer.
  fn text_buffer(&mut self, id: Id) -> &mut String {
    self.text_fields.entry(id).or_default();
    return self.text_fields.get_mut(&id).unwrap();
  }

  /// Clears all temp field buffers.
  fn clear_buffers(&mut self) {
    self.text_fields.clear();
  }

  /// Changes the view, and clears text buffers.
  fn switch_to(&mut self, view: View) {
    self.view = view;
    self.clear_buffers();
  }

  /// Editable list of things.
  fn editable_vec<F, T: Clone + PartialEq + ToString + FromStr>(
    &mut self,
    ui: &mut Ui,
    finder: F
  ) -> Rect
  where
    <T as FromStr>::Err: Debug,
    F: Fn(&mut Self) -> &mut Vec<T>
  {
    let clone = finder(self).clone();
    egui::Grid::new(ui.id()).show(ui, |ui| {
      for (i, x) in clone.iter().enumerate() {
        ui.label(&x.to_string());
        if ui.button("✕").clicked() {
          finder(self).remove(i);
        }
        ui.end_row();
      }
      let field_id = ui.next_auto_id();
      let buf = self.text_buffer(field_id);
      ui.text_edit_singleline(buf);
      if ui.button("add").clicked() {
        if let Ok(k) = buf.parse::<T>() {
          finder(self).push(k);
        }
        self.text_buffer(field_id).clear();
      }
      ui.end_row();
    });
    return ui.min_rect();
  }

  /// Render function for the menu bar.
  fn show_menu(&mut self, ctx: &egui::Context, ui: &mut Ui) {
    egui::menu::bar(ui, |ui| {
      // suite menu
      ui.menu_button("Suite", |ui| {
        if ui.button("Save").clicked() {
          self.try_run(ui, Gui::save_suite);
        }
        if ui.button("Save as...").clicked() {
          self.try_run(ui, Gui::save_suite_as);
        }
        if ui.button("Load").clicked() {
          self.try_run(ui, Gui::load_suite);
        }
      });
      // decks menu
      ui.menu_button("Decks", |ui| {
        if ui.button("View/edit decks").clicked() {
          self.view = View::Decks;
        }
        if ui.button("Add...").clicked() {
          self.try_run(ui, Gui::add_decks);
        }
      });
      // generates sub-menu solver buttons
      let btn = |
        ui: &mut Ui,
        opt: Option<Uuid>,
        lbl: &str,
        tgt: &mut Option<Uuid>
      | {
        let label = format!(
          "{}{}",
          lbl,
          if opt == *tgt { " (selected)" } else { "" },
        );
        if ui.button(label).clicked() {
          *tgt = opt
        }
      };
      // solvers menu
      ui.menu_button("Solvers", |ui| {
        if ui.button("View/edit solvers").clicked() {
          self.view = View::Solvers;
        }
        if ui.button("Add solver binary...").clicked() {
          self.try_run(ui, Gui::add_solver_bin);
        }
        if ui.button("Add F06 directory...").clicked() {
          self.try_run(ui, Gui::add_solver_dir);
        }
        ui.menu_button("Set reference solver", |ui| {
          btn(ui, None, "<none>", &mut self.state.ref_solver);
          for (u, s) in self.state.solvers.iter() {
            btn(ui, Some(*u), s.nickname.as_str(), &mut self.state.ref_solver);
          }
        });
        ui.menu_button("Set solver under test", |ui| {
          btn(ui, None, "<none>", &mut self.state.test_solver);
          for (u, s) in self.state.solvers.iter() {
            btn(ui, Some(*u), s.nickname.as_str(), &mut self.state.test_solver);
          }
        });
      });
      // criteria sets menu
      ui.menu_button("Criteria sets", |ui| {
        if ui.button("Edit criteria sets").clicked() {
          self.view = View::CriteriaSets;
        }
      });
      // advanced stuff
      ui.menu_button("Advanced", |ui| {
        // dark mode toggler
        let dark_mode = ctx.style().visuals.dark_mode;
        if dark_mode {
          if ui.button("Change to light mode").clicked() {
            ctx.set_visuals(Visuals::light());
          }
        } else if ui.button("Change to dark mode").clicked() {
          ctx.set_visuals(Visuals::dark());
        }
        // dump app state
        if ui.button("Dump app state").clicked() {
          info!("User-requested dump of app state:\n{:#?}", self);
        }
        // toggle gui debug
        let guidebug = if ctx.debug_on_hover() { "ON" } else { "OFF" };
        if ui.button(&format!("GUI debug {}", guidebug)).clicked() {
          ctx.set_debug_on_hover(!ctx.debug_on_hover());
        }
      });
    });
  }

  /// Render function for the global decks list.
  fn show_decks(&mut self, ctx: &egui::Context) {
    // one per deck
    let deck_data = self.state.decks_by_name()
      .map(|(u, d, r)| (u, d.clone(), r.cloned()))
      .collect::<Vec<_>>();
    egui::CentralPanel::default().show(ctx, |ui| {
      self.show_menu(ctx, ui);
      let heading_height = ui.text_style_height(&TextStyle::Heading);
      let body_height = ui.text_style_height(&TextStyle::Body);
      let mut cells = Layout::left_to_right(Align::Center);
      let ndecks = deck_data.len();
      if deck_data.is_empty() {
        ui.columns(3, |cols| {
          cols[1].horizontal_centered(|ui| {
            egui::Grid::new("no_decks_grid").show(ui, |ui| {
              ui.strong("No decks in current suite.");
              ui.end_row();
              ui.horizontal(|ui| {
                ui.label("Maybe");
                if ui.button("add some").clicked() {
                  self.try_run(ui, Gui::add_decks);
                }
                ui.label("or");
                if ui.button("load a suite file").clicked() {
                  self.try_run(ui, Gui::load_suite);
                }
                ui.label("?");
              });
              ui.end_row();
            })
          });
        });
      } else {
        ui.vertical_centered(|ui| {
          ui.strong("Decks in current suite:");
        });
        cells.main_wrap = false;
        TableBuilder::new(ui)
          .vscroll(true)
          .auto_shrink(true)
          .striped(true)
          .cell_layout(cells)
          .column(Column::remainder().resizable(true))
          .column(Column::auto().resizable(true))
          .column(Column::auto().resizable(true))
          .column(Column::auto().resizable(true))
          .column(Column::auto().resizable(true))
          .header(heading_height, |mut header| {
            header.col(|ui| { ui.heading("File name"); });
            header.col(|ui| { ui.heading("Status"); });
            header.col(|ui| { ui.heading("Reference run"); });
            header.col(|ui| { ui.heading("Test run"); });
            header.col(|ui| { ui.heading("Actions"); });
          })
          .body(|body| {
            body.rows(body_height, ndecks, |mut row| {
              let (uuid, deck, results) = deck_data.get(row.index()).unwrap();
              // filename
              row.col(|ui| { ui.label(deck.name()); });
              // status
              row.col(|ui| if deck.in_file.is_file() {
                ui.label("Ready");
              } else {
                ui.add(egui::Label::new(
                  WidgetText::from("Missing").strong().color(Color32::RED))
                );
              });
              // results
              if let Some(res) = results {
                let lblres = |ui: &mut Ui, res: &Result<_, String>| {
                  let (text, color) = match res {
                    Ok(_) => ("Finished".to_owned(), Color32::DARK_GREEN),
                    Err(e) => (format!("Error: {}", e), Color32::RED),
                  };
                  ui.add(egui::Label::new(
                    WidgetText::from(&text).color(color))
                  );
                };
                // reference run
                row.col(|ui| lblres(ui, &res.ref_f06));
                // test run
                row.col(|ui| lblres(ui, &res.test_f06));
              } else {
                // reference run
                row.col(|ui| { ui.label("Not yet run"); });
                // test run
                row.col(|ui| { ui.label("Not yet run"); });
              }
              // actions
              row.col(|ui| {
                ui.horizontal(|ui| {
                  if ui.button("Configure").clicked() {
                    self.switch_to(View::Deck(*uuid));
                  }
                  if ui.button("Remove").clicked() {
                    self.state.suite.decks.remove(uuid);
                    self.suite_clean = false;
                  }
                });
              });
            });
        });
      }
    });
  }

  /// Aux function to render specifier inputs.
  fn show_specifier<F, T: Clone + PartialEq + ToString + FromStr>(
    &mut self,
    ui: &mut Ui,
    finder: F
  ) where
    <T as FromStr>::Err: Debug,
    F: Fn(&mut Self) -> &mut Specifier<T>
  {
    ui.horizontal(|ui| {
      let tgt = finder(self);
      let clone = tgt.clone();
      ComboBox::from_id_source(ui.next_auto_id())
        .selected_text(format!("{}", clone.get_type()))
        .show_ui(ui, |ui| {
          let types = [
            SpecifierType::All, SpecifierType::List, SpecifierType::AllExcept
          ];
          for new_type in types {
            ui.selectable_value(tgt, clone.with_type(new_type), new_type.name());
          }
        });
      match clone {
        Specifier::All => {},
        Specifier::List(_) | Specifier::AllExcept(_) => {
          self.editable_vec(ui, |s| finder(s).inner_vec_mut().unwrap());
        },
      };
    });
  }

  /// Render function for a single deck, its extractions, etcetera.
  fn show_deck(&mut self, ctx: &egui::Context, uuid: Uuid) {
    if let Some((deck_ref, results_ref)) = self.state.get_deck(uuid) {
      let deck = deck_ref.clone();
      let _results = results_ref.cloned();
      let exns_ui = |ui: &mut Ui| {
        ui.vertical_centered(|ui| {
          ui.strong("Deck extractions:");
          if ui.button("Add new").clicked() {
            if let Some(dref) = self.state.suite.decks.get_mut(&uuid) {
              dref.extractions.push((Extraction::default(), None));
            }
          }
          let heading_height = ui.text_style_height(&TextStyle::Heading);
          let body_height = ui.text_style_height(&TextStyle::Body);
          TableBuilder::new(ui)
            .vscroll(true)
            .auto_shrink(true)
            .striped(true)
            .column(Column::auto())
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .header(heading_height, |mut header| {
              header.col(|ui| { ui.label("nº"); });
              header.col(|ui| { ui.label("subcases"); });
              header.col(|ui| { ui.label("nodes"); });
              header.col(|ui| { ui.label("elements"); });
              header.col(|ui| { ui.label("criteria"); });
            })
            .body(|mut body| {
              for (i, (exn, crit)) in deck.extractions.iter().enumerate() {
                let max_exn_lens = [
                  exn.subcases.inner_vec().map_or(0, |v| v.len()),
                  exn.grid_points.inner_vec().map_or(0, |v| v.len()),
                  exn.elements.inner_vec().map_or(0, |v| v.len())
                ].into_iter().max().unwrap();
                let row_height = max_exn_lens.max(1) as f32 * body_height * 2.;
                body.row(row_height, |mut row| {
                  row.col(|ui| { ui.label(&i.to_string()); });
                  row.col(|ui| {
                    self.show_specifier(ui, |s| &mut s.state
                      .suite.decks.get_mut(&uuid).expect("deck UUID missing!")
                      .extractions.get_mut(i).expect("bad extraction index!")
                      .0.subcases
                    );
                  });
                  row.col(|ui| {
                    self.show_specifier(ui, |s| &mut s.state
                      .suite.decks.get_mut(&uuid).expect("deck UUID missing!")
                      .extractions.get_mut(i).expect("bad extraction index!")
                      .0.grid_points
                    );
                  });
                  row.col(|ui| {
                    self.show_specifier(ui, |s| &mut s.state
                      .suite.decks.get_mut(&uuid).expect("deck UUID missing!")
                      .extractions.get_mut(i).expect("bad extraction index!")
                      .0.elements
                    );
                  });
                  row.col(|ui| {
                    ComboBox::from_id_source(ui.next_auto_id())
                      .selected_text(crit.map_or(
                        "<none>".to_owned(),
                        |u| self.state.suite.criteria_sets
                          .get(&u)
                          .map(|c| c.name.clone())
                          .expect("critset UUID missing"))
                      ).show_ui(ui, |ui| {
                        let crit_mut = &mut self.state.suite.decks.get_mut(&uuid).unwrap().extractions.get_mut(i).unwrap().1;
                        ui.selectable_value(crit_mut, None, "<none>");
                        let critsets = self.state.suite.criteria_sets.iter();
                        for (uuid, crit) in critsets {
                          ui.selectable_value(crit_mut, Some(*uuid), &crit.name);
                        }
                      });
                  });
                })
              }
            })
        })
      };
      egui::TopBottomPanel::bottom("deck_extractions")
        .resizable(true)
        .show_separator_line(true)
        .show(ctx, exns_ui);
      egui::CentralPanel::default().show(ctx, |ui| {
        self.show_menu(ctx, ui);
      });
    } else {
      egui::CentralPanel::default().show(ctx, |ui| {
        self.show_menu(ctx, ui);
        log::error!("Tried to go to deck with invalid UUID!");
        ui.label("Invalid deck UUID!");
      });
    }
  }
}

impl eframe::App for Gui {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    //if cfg!(debug_assertions) {
    //  ctx.set_debug_on_hover(true);
    //}
    match self.view {
      View::Decks => self.show_decks(ctx),
      View::Solvers => todo!(),
      View::CriteriaSets => todo!(),
      View::Logs => todo!(),
      View::Deck(uuid) => self.show_deck(ctx, uuid),
      View::Solver(_) => todo!(),
      View::CriteriaSet(_) => todo!(),
    };
  }
}
