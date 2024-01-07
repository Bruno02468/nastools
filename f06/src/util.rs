//! This module implements utility functions without much need for defining
//! context or not enough of it to warrant them having their own modules.

use std::cell::Cell;

use serde::{Serialize, Deserialize};

use crate::elements::ElementType;

/// Decodes a Nastran-format floating point number. Hyper-lenient and doesn't
/// require pulling a whole regex library.
pub(crate) fn decode_nasfloat(s: &str) -> Option<f64> {
  // mantissa start/end, exponent start/end
  let mut ixs: [usize; 4] = [0, 0, 0, 0];
  // 0-1 = looking for mantissa start/end, 2-3 = looking for exponent start/end
  let step: Cell<usize> = 0.into();
  let mut mark = |i| { ixs[step.get()] = i; step.replace(step.get() + 1); };
  let mut seen_chars: usize = 0;
  for (i, c) in s.chars().enumerate() {
    seen_chars += 1;
    match (step.get() % 2, c.is_numeric() || c == '.', c == '+' || c == '-') {
      // looking for number start. nothing yet. keep looking.
      (0, false, false) => continue,
      // looking for number start, found something. mark it and look for end.
      (0, _, _) => mark(i),
      // looking for number end, saw number/dot. keep looking.
      (1, true, _) => continue,
      // looking for number end, saw not numerical/dot/sign. mark end.
      (1, false, false) => mark(i),
      // looking for number end, saw sign. mark end, mark start.
      (1, _, true) => { mark(i); mark(i); },
      // should be unreachable
      _ => panic!("unreachable branch 1 in decoding nasfloat \"{}\"", s)
    };
    if step.get() > 3 { break; }
  }
  // handle empty string
  if seen_chars == 0 {
    return None;
  }
  // handle end at end-of-string
  if step.get() % 2 == 1 {
    mark(seen_chars);
  }
  let mantissa = || s[ixs[0]..ixs[1]].parse::<f64>().ok();
  let exponent = || s[ixs[2]..ixs[3]].parse::<i32>().ok();
  match step.get() {
    // never found mantissa
    0 => return None,
    // only found a mantissa
    2 => return mantissa(),
    // found mantissa and exponent
    4 => return Some(mantissa()? * 10.0_f64.powi(exponent()?)),
    // should be unreachable
    _ => panic!("unreachable branch 2 in returning nasfloat \"{}\"", s)
  };
}

/// A line field as decoded.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub(crate) enum LineField<'s> {
  /// Managed to parse an integer out of it.
  Integer(isize),
  /// Managed to parse a real out of it.
  Real(f64),
  /// Field is a single character.
  Character(char),
  /// Field is an element type.
  ElementType(ElementType),
  /// Couldn't parse it.
  NoIdea(&'s str)
}

impl<'s> LineField<'s> {
  /// Parses a single field into a LineField.
  fn parse(s: &'s str) -> Self {
    if let Ok(i) = s.parse::<isize>() {
      return Self::Integer(i);
    }
    if let Ok(x) = s.parse::<f64>()/*.or(decode_nasfloat(s))*/ {
      return Self::Real(x);
    }
    if s.len() == 1 {
      return Self::Character(s.chars().nth(0).unwrap());
    }
    for cand in ElementType::all() {
      if s == cand.name() {
        return Self::ElementType(*cand);
      }
    }
    return Self::NoIdea(s);
  }
}

/// Breaks down a line into an iterator of fields.
pub(crate) fn line_breakdown(
  s: &str
) -> impl Iterator<Item = LineField<'_>> {
  return s.split(' ')
    .filter(|subs| !subs.is_empty())
    .map(LineField::parse);
}

/// Gets a certain number of reals from a line.
pub(crate) fn extract_reals<const N: usize>(line: &str) -> Option<[f64; N]> {
  let mut arr: [f64; N] = [0.0; N];
  let mut found = 0;
  for field in line_breakdown(line) {
    match field {
      LineField::Real(x) if found < N => {
        arr[found] = x;
        found += 1;
      },
      LineField::Real(_) if found == N => {
        return None;
      },
      _ => continue
    }
  }
  if found == N {
    return Some(arr);
  } else {
    return None;
  }
}

/// Gets the N-th integer in a line.
pub(crate) fn nth_integer(line: &str, n: usize) -> Option<isize> {
  return line_breakdown(line)
    .filter_map(|field| {
      if let LineField::Integer(x) = field {
        return Some(x)
      } else {
        None
      }
    }).nth(n);
}

/// Gets the N-th string in a line.
pub(crate) fn nth_string(line: &str, n: usize) -> Option<&str> {
  return line_breakdown(line)
    .filter_map(|field| {
      if let LineField::NoIdea(s) = field {
        return Some(s)
      } else {
        None
      }
    }).nth(n);
}

/// Gets the N-th element type in a line.
pub(crate) fn nth_etype(line: &str, n: usize) -> Option<ElementType> {
  return line_breakdown(line)
    .filter_map(|field| {
      if let LineField::ElementType(etype) = field {
        return Some(etype)
      } else {
        None
      }
    }).nth(n);
}
