//! This module implements comparison of blocks and the data within. The
//! `f06diff` tool is an example of this module's capabilities.

use std::collections::BTreeSet;
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Serialize, Deserialize};

use crate::prelude::*;

/// This enumeration is a "shallow" comparison of blocks -- the data isn't
/// compared, it's just to see what the blocks have in common, structurally
/// speaking.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlockCompatibility {
  /// The blocks aren't even of the same type.
  DifferentType,
  /// The blocks aren't the same subcase.
  DifferentSubcase,
  /// The blocks don't have the same column indexes.
  DifferentColumns,
  /// The blocks have no row indexes in common.
  NoCommonRows,
  /// The blocks are compatible for data comparison.
  Compatible {
    /// The rows the blocks have in common.
    common_rows: BTreeSet<NasIndex>,
    /// The rows one block has but the other one doesn't.
    disjunction: BTreeSet<NasIndex>
  }
}

impl From<(&FinalBlock, &FinalBlock)> for BlockCompatibility {
  fn from((a, b): (&FinalBlock, &FinalBlock)) -> Self {
    if a.block_type != b.block_type {
      return Self::DifferentType;
    }
    if a.subcase != b.subcase {
      return Self::DifferentSubcase;
    }
    let aci = a.col_indexes.keys().copied().collect::<BTreeSet<_>>();
    let bci = b.col_indexes.keys().copied().collect::<BTreeSet<_>>();
    if aci != bci {
      return Self::DifferentColumns;
    }
    let ari = a.row_indexes.keys().copied().collect::<BTreeSet<_>>();
    let bri = b.row_indexes.keys().copied().collect::<BTreeSet<_>>();
    let ixn = &ari & &bri;
    let dxn = &ari ^ &bri;
    if ixn.is_empty() {
      return Self::NoCommonRows;
    }
    return Self::Compatible { common_rows: ixn, disjunction: dxn };
  }
}

/// What to do when there's a row disunction (i.e. there are some rows that
/// appear in one block but not another.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisjunctionBehaviour {
  /// Skip the disjunct row, do not include them in the comparison.
  Skip,
  /// Assume an all-zero row where it's missing.
  AssumeZeroes,
  /// Flag the row and column.
  Flag
}

impl Default for DisjunctionBehaviour {
  fn default() -> Self {
    return Self::AssumeZeroes;
  }
}

impl FromStr for DisjunctionBehaviour {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    return Self::all()
      .iter()
      .copied()
      .find(|v| s.eq_ignore_ascii_case(v.small_lc_name()))
      .ok_or(());
  }
}

impl ValueEnum for DisjunctionBehaviour {
  fn value_variants<'a>() -> &'a [Self] {
    return Self::all();
  }

  fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
    return Some(self.small_lc_name().into());
  }
}

impl DisjunctionBehaviour {
  /// Returns all variants.
  pub const fn all() -> &'static [Self] {
    return &[
      Self::Skip,
      Self::AssumeZeroes,
      Self::Flag
    ];
  }

  /// Returns a small name for the variant (lower-case).
  pub const fn small_lc_name(&self) -> &'static str {
    return match self {
      DisjunctionBehaviour::Skip => "skip",
      DisjunctionBehaviour::AssumeZeroes => "zero",
      DisjunctionBehaviour::Flag => "flag",
    };
  }
}

/// Value testing/comparison criteria.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Criteria {
  /// Test an absolute value difference?
  difference: Option<f64>,
  /// Test a big-to-small ratio/
  ratio: Option<f64>,
  /// Check for NaNs?
  nan: bool,
  /// Check for infinities?
  inf: bool,
  /// Check for differing signs?
  sig: bool
}

impl Criteria {
  /// Checks a pair of values against this set of criteria.
  pub fn check(&self, a: f64, b: f64) -> Option<FlagReason> {
    // check for NaNs
    if self.nan && (a.is_nan() || b.is_nan()) {
      return Some(FlagReason::NaN);
    }
    // check for infinities
    if self.inf && (a.is_infinite() || b.is_infinite()) {
      return Some(FlagReason::Infinity);
    }
    // check signs
    if self.sig && (a.signum() != b.signum()) {
      return Some(FlagReason::Signs);
    }
    // check difference
    if let Some(eps) = self.difference {
      let diff = (a-b).abs();
      if diff > eps {
        return Some(FlagReason::Difference {
          abs_difference: diff,
          max_epsilon: eps
        });
      }
    }
    // check ratio
    if let Some(max_ratio) = self.ratio {
      let (big, small) = if a >= b { (a, b) } else { (b, a) };
      let rat = big/small;
      if rat > max_ratio {
        return Some(FlagReason::Ratio { big_to_small: rat, max_ratio })
      }
    }
    // nothing? no flag
    return None;
  }
}

/// Holds a found value in two data blocks.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FoundValues {
  /// The row index.
  row: NasIndex,
  /// The column index.
  col: NasIndex,
  /// The value in block A.
  val_a: F06Number,
  /// The value in block B.
  val_b: F06Number
}

/// The reason a value was flagged.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FlagReason {
  /// Flagged due to an absolute difference.
  Difference {
    /// The absolute-value difference between the numbers.
    abs_difference: f64,
    /// The exceeded epsilon value.
    max_epsilon: f64
  },
  /// Flagged due to an exceeded ratio.
  Ratio {
    /// The ratio between the larger and the smaller number.
    big_to_small: f64,
    /// The max ratio exceeded.
    max_ratio: f64
  },
  /// Flagged due to being a NaN.
  NaN,
  /// Flagged due to there being an infinity.
  Infinity,
  /// Signs differ!
  Signs,
  /// Row is misisng in one of the blocks.
  Disjunction
}

/// This structure holds a flagged difference in data.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct FlaggedPosition {
  /// The flagged values and their positions.
  values: FoundValues,
  /// The reason for flagging.
  reason: FlagReason
}

/// This structure holds the necessary data to diff data blocks. It could be
/// made parallel, but there's been no need to make this parallel... for now.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct DataDiffer {
  /// The value-flagging criteria.
  criteria: Criteria,
  /// What to do when doing disjunct lines?
  dxn_behaviour: DisjunctionBehaviour
}

impl DataDiffer {
  /// Instantiate a new DataDiffer with the given settings.
  pub fn new(
    criteria: Criteria,
    dxn_behaviour: DisjunctionBehaviour
  ) -> Self {
    return Self { criteria, dxn_behaviour };
  }

  /// Diff two data blocks and return flagged positions.
  pub fn compare<'a>(
    &'a self,
    a: &'a FinalBlock,
    b: &'a FinalBlock
  ) -> Result<impl Iterator<Item = FlaggedPosition> + 'a, BlockCompatibility> {
    let comp = BlockCompatibility::from((a, b));
    if !matches!(comp, BlockCompatibility::Compatible { .. }) {
      return Err(comp);
    }
    let get = |
      s: &FinalBlock,
      r: &NasIndex,
      c: &NasIndex
    | -> Result<Option<f64>, FlagReason> {
      if s.row_indexes.contains_key(r) {
        return Ok(Some(s.get(*r, *c).unwrap().into()));
      } else {
        match self.dxn_behaviour {
          DisjunctionBehaviour::Skip => return Ok(None),
          DisjunctionBehaviour::AssumeZeroes => return Ok(Some(0.0)),
          DisjunctionBehaviour::Flag => return Err(FlagReason::Disjunction),
        }
      }
    };
    return Ok(
      a.row_indexes
        .keys().copied()
        .zip(a.col_indexes.keys().copied())
        .filter_map(move |(r, c)| {
          let mut fv = FoundValues {
            row: r,
            col: c,
            val_a: 0.0.into(),
            val_b: 0.0.into()
          };
          match (get(a, &r, &c), get(b, &r, &c)) {
            // got both values
            (Ok(Some(x)), Ok(Some(y))) => {
              fv.val_a = x.into();
              fv.val_b = y.into();
              return self.criteria.check(x, y)
                .map(|fr| FlaggedPosition { values: fv, reason: fr });
            },
            (Ok(_), Ok(None)) | (Ok(None), Ok(_)) => {
              // got both values but at least one skip
              return None;
            }
            (_, Err(fr)) | (Err(fr), _) => {
              // at least one disjunction
              return Some(FlaggedPosition { values: fv, reason: fr });
            }
          }
        })
    );
  }
}
