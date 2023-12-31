//! This module implements differences between flavours of text output between
//! solvers and their varied configurtions/solution types.

/// The different supported solvers.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Solver {
  /// The MYSTRAN solver originally developed by Dr. Bill Case.
  Mystran,
  /// The Simcenter Nastran solver, formerly known as NX Nastran.
  Simcenter
}

/// The known solution types.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum SolType {
  /// Linear static analysis, also known as SOL STATIC or SOL 101.
  LinearStatic,
  /// Eigenvalue/modes analyssis, also known as SOL MODES or SOL 103.
  Eigenvalue,
  /// Linear static solutions with differential stiffness.
  LinearStaticDiffStiff,
  /// Linear buckling analysis, also known as SOL BUCKLING or SOL 105.
  LinearBuckling
}

impl Default for SolType {
  fn default() -> Self {
    return Self::LinearStatic;
  }
}

impl TryFrom<usize> for SolType {
  type Error = ();

  fn try_from(sol: usize) -> Result<Self, Self::Error> {
    return match sol {
      1 | 101 => Ok(Self::LinearStatic),
      3 | 103 => Ok(Self::Eigenvalue),
      4 | 104 => Ok(Self::LinearStaticDiffStiff),
      5 | 105 => Ok(Self::LinearBuckling),
      _ => Err(())
    }
  }
}
