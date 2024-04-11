//! This is a stand-alone program to run and test MYSTRAN against other
//! solvers, using their .F06 outputs and user-set criteria.

#![warn(missing_docs)] // almost sure this is default but whatever
#![warn(clippy::missing_docs_in_private_items)] // sue me
#![allow(clippy::needless_return)] // i'll never forgive rust for this

pub(crate) mod running;
pub(crate) mod suite;

fn main() {
  println!("Hello, world!");
}
