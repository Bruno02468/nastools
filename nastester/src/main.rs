//! This is a stand-alone program to run and test MYSTRAN against other
//! solvers, using their .F06 outputs and user-set criteria.

#![warn(missing_docs)] // almost sure this is default but whatever
#![warn(clippy::missing_docs_in_private_items)] // sue me
#![allow(clippy::needless_return)] // i'll never forgive rust for this
#![allow(dead_code)]

use log::LevelFilter;

use crate::gui::Gui;

pub(crate) mod app;
pub(crate) mod gui;
pub(crate) mod results;
pub(crate) mod running;
pub(crate) mod suite;

#[cfg(debug_assertions)]
/// Default log level for debug builds.
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Debug;

#[cfg(not(debug_assertions))]
/// Default log level for release builds.
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() -> Result<(), eframe::Error> {
  env_logger::builder().filter_level(DEFAULT_LOG_LEVEL).init();
  let native_options = eframe::NativeOptions {
    // does this even work?
    persist_window: true,
    // linux window centering puts it in the wrong display sometimes
    #[cfg(target_os = "macos")]
    centered: true,
    #[cfg(target_os = "linux")]
    centered: false,
    #[cfg(target_os = "windows")]
    centered: true,
    // let's not touch the rest
    ..Default::default()
  };
  return eframe::run_native(
    &format!("nastester {}", env!("CARGO_PKG_VERSION")),
    native_options,
    Box::new(|cc| Box::new(Gui::new(cc))),
  );
}
