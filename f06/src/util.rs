//! This module implements utility functions without much need for defining
//! context or not enough of it to warrant them having their own modules.

use std::cell::Cell;

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
