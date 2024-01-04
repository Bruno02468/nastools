//! This module implements differences between flavours of text output between
//! solvers and their varied configurtions/solution types.

use std::fmt::Display;

/// The different supported solvers.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Solver {
  /// The default: unknown solver -- we'll do our best to decode.
  Unknown,
  /// The MYSTRAN solver originally developed by Dr. Bill Case.
  Mystran,
  /// The Simcenter Nastran solver, formerly known as NX Nastran.
  Simcenter
}

impl Default for Solver {
  fn default() -> Self {
    return Self::Unknown;
  }
}

impl Display for Solver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", self.name());
  }
}

impl Solver {
  /// Returns a constant display name for the solver.
  pub const fn name(&self) -> &'static str {
    return match self {
      Solver::Unknown => "unknown",
      Solver::Mystran => "MYSTRAN",
      Solver::Simcenter => "Simcenter Nastran"
    };
  }
}

/// The known solution types.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum SolType {
  /// Unknown solution type.
  Unknown,
  /// Linear static analysis, also known as SOL STATIC or SOL 101.
  LinearStatic,
  /// Eigenvalue/modes analyssis, also known as SOL MODES or SOL 103.
  Eigenvalue,
  /// Linear static solutions with differential stiffness.
  LinearStaticDiffStiff,
  /// Linear buckling analysis, also known as SOL BUCKLING or SOL 105.
  LinearBuckling,
  /// Nonlinear static analysis, also known as SOL NLSTATIC or SOL 106.
  NonLinearStatic
}

impl Default for SolType {
  fn default() -> Self {
    return Self::Unknown;
  }
}

impl From<SolType> for usize {
  fn from(value: SolType) -> Self {
    return match value {
      SolType::Unknown => 0,
      SolType::LinearStatic => 101,
      SolType::Eigenvalue => 103,
      SolType::LinearStaticDiffStiff => 104,
      SolType::LinearBuckling => 105,
      SolType::NonLinearStatic => 106,
    };
  }
}

impl From<usize> for SolType {
  fn from(sol: usize) -> Self {
    return match sol {
      0 => Self::Unknown,
      1 | 101 => Self::LinearStatic,
      3 | 103 => Self::Eigenvalue,
      4 | 104 => Self::LinearStaticDiffStiff,
      5 | 105 => Self::LinearBuckling,
      106 => Self::NonLinearStatic,
      _ => Self::Unknown
    };
  }
}
