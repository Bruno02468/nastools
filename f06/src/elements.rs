//! This module defines the different kinds of elements that can be found in
//! Nastran output so that output fields can be taken generically over elements
//! and so the code is easier to expand.

use std::fmt::Debug as DebugTrait;

use serde::{Serialize, Deserialize};

use crate::fields::SMALL_FIELD_BYTES;

/// Broadly-defined element categories.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ElementCategory {
  /// Rigid-body elements, like RBE2.
  RigidBody,
  /// Scalar spring elements, like ELAS1.
  ScalarSpring,
  /// Bushing elements, like BUSH.
  Bushing,
  /// Rod elements, like ROD.
  Rod,
  /// Bar elements, like BAR.
  Bar,
  /// Plate elements, like QUAD4.
  Plate,
  /// Solid elements, like HEXA.
  Solid
}

/// Elements must implement this object-safe trait.
pub trait Element: Clone + DebugTrait {
  /// Number of grid points this element is connected to.
  const ELGP: u8;
  /// Type of element.
  const ELTYPE: ElementCategory;
  /// The name of the element, fitting in at most eight characters.
  const ELNAME: [u8; SMALL_FIELD_BYTES];
  
  /// Returns the ID of the element.
  fn eid(&self) -> usize;
}
