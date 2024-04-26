//! This defines subroutines to run decks and do test runs.

use std::error::Error;
use core::fmt::Display;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use f06::prelude::*;
use serde::{Deserialize, Serialize};
use subprocess::{ExitStatus, Popen, PopenConfig, PopenError};

use crate::suite::*;

#[cfg(target_os = "macos")]
/// Extensions for binary files.
pub(crate) const BINARY_EXTENSIONS: &[&str] = &[];
#[cfg(target_os = "linux")]
/// Extensions for binary files.
pub(crate) const BINARY_EXTENSIONS: &[&str] = &[];
#[cfg(target_os = "windows")]
/// Extensions for binary files.
pub(crate) const BINARY_EXTENSIONS: &[&str] = &["exe"];

/// Lower-case F06 extension.
const F06_LOWER: &str = "f06";

/// Upper-case F06 extension.
const F06_UPPER: &str = "F06";

/// This is how we run a solver, if at all, to acquire an F06 file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum RunMethod {
  /// The F06 file is directly imported from a directory, containing the .F06
  /// files with the same base name as the decks.
  ImportFromDir(PathBuf),
  /// A solver is run passing the deck as an argument, and the F06 is got from
  /// reading the same
  RunSolver(PathBuf)
}

/// These are the errors that can come up when running a solver to get the F06
/// output.
#[derive(Debug, derive_more::From)]
pub(crate) enum RunError {
  /// The F06 was not found at its supposed location.
  MissingF06(PathBuf),
  /// The F06 was found but could not be read.
  UnreadableF06(PathBuf, Box<dyn Error>),
  /// The solver binary is missing.
  MissingSolver(PathBuf),
  /// The solver subprocess exited with a non-zero code.
  SolverFailed(String, Option<u32>),
  /// Couldn't process a path to get basenames, extensions, etc.
  PathError,
  /// Two matching files exist in the directory.
  ExtensionMixup,
  /// The F06 parser failed for some reason.
  #[from]
  IoError(std::io::Error),
  /// Couldn't create a temp dir.
  TempdirCreationFailed,
  /// Coulndn't spawn a subprocess.
  #[from]
  SubprocessFailed(PopenError)
}

impl Display for RunError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      RunError::MissingF06(p) => write!(
        f,
        "missing F06 file at {}",
        p.display()
      ),
      RunError::UnreadableF06(p, e) => write!(
        f,
        "could not read F06 file at {}, reason: {}",
        p.display(),
        e
      ),
      RunError::MissingSolver(p) => write!(
        f,
        "missing solver binary at {}",
        p.display()
      ),
      RunError::SolverFailed(s, Some(c)) => write!(
        f,
        "solver \"{}\" finished with non-zero exit code {}",
        s,
        c
      ),
      RunError::SolverFailed(s, None) => write!(
        f,
        "solver \"{}\" failed to run and didn't even return a code",
        s,
      ),
      RunError::PathError => write!(f, "path processing error"),
      RunError::ExtensionMixup => write!(f, "both .f06 and .F06 exist"),
      RunError::IoError(ioe) => write!(f, "I/O error: {}", ioe),
      RunError::TempdirCreationFailed => write!(f, "tempdir creation failed"),
      RunError::SubprocessFailed(e) => write!(f, "subprocess error: {}", e),
    };
  }
}

impl RunError {
  /// Returns a shorter error message.
  pub(crate) fn short_msg(&self) -> &'static str {
    return match self {
      RunError::MissingF06(_) => "missing F06 file",
      RunError::UnreadableF06(_, _) => "couldn't read F06 file",
      RunError::MissingSolver(_) => "missing solver binary",
      RunError::SolverFailed(_, Some(_)) => "solver failed to run",
      RunError::SolverFailed(_, None) => "solver crashed",
      RunError::PathError => "path processing error",
      RunError::ExtensionMixup => "both .f06 and .F06 exist",
      RunError::IoError(_) => "I/O error",
      RunError::TempdirCreationFailed => "tempdir creation failed",
      RunError::SubprocessFailed(_) => "subprocess error",
    };
  }
}

/// This is a named "F06 acquisition method". A solver, for short.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RunnableSolver {
  /// The kind of solver. Must be supported by the F06 library.
  pub(crate) kind: Solver,
  /// The "nickname" for this solver, so you can tell versions apart.
  pub(crate) nickname: String,
  /// The method through which we actually get an F06.
  pub(crate) method: RunMethod
}

impl RunnableSolver {
  /// Runs this solver and returns an F06 output.
  pub(crate) fn make_f06(&self, deck: &Deck) -> Result<F06File, RunError> {
    let basename = deck.in_file.file_name().ok_or(RunError::PathError)?;
    /// This function extracts the F06 file from a directory.
    fn do_dir(d: &Path, basename: &OsStr) -> Result<F06File, RunError> {
      let mut lower = d.join(basename);
      lower.set_extension(F06_LOWER);
      let mut upper = d.join(basename);
      upper.set_extension(F06_UPPER);
      let f06path = match (lower.exists(), upper.exists()) {
        (true, true) => return Err(RunError::ExtensionMixup),
        (false, false) => return Err(RunError::MissingF06(d.to_path_buf())),
        (true, false) => lower,
        (false, true) => upper,
      };
      let mut file = f06::parser::OnePassParser::parse_file(f06path)?;
      file.merge_blocks(true);
      file.sort_all_blocks();
      return Ok(file);
    }
    match &self.method {
      RunMethod::ImportFromDir(d) => return do_dir(d, basename),
      RunMethod::RunSolver(bin) => {
        let tmp = tempdir::TempDir::new("nastester_run_")
          .map_err(|_| RunError::TempdirCreationFailed)?;
        let pc = PopenConfig {
          stdin: subprocess::Redirection::None,
          stdout: subprocess::Redirection::None,
          stderr: subprocess::Redirection::None,
          executable: Some(bin.clone().into_os_string()),
          cwd: Some(tmp.path().as_os_str().to_owned()),
          ..Default::default()
        };
        let mut proc = Popen::create(&[bin, &deck.in_file], pc)?;
        let code = proc.wait()?;
        match code {
          ExitStatus::Exited(0) => {
            return do_dir(tmp.path(), basename);
          },
          ExitStatus::Exited(i) => return Err(
            RunError::SolverFailed(self.nickname.clone(), Some(i))
          ),
          _ => return Err(RunError::SolverFailed(self.nickname.clone(), None))
        };
      },
    };
  }
}

/// These are testing settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TestSettings {
  /// Keep the parsed F06 results?
  pub(crate) keep_f06_mem: bool,
  /// Max flagged indices per deck.
  pub(crate) max_flags: usize
}
