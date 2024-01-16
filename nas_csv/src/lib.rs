//! This library implements a highly-configurable CSV format to convert Nastran
//! output to. Primarily meant for use by the `f06csv` tool, i.e. F06 to CSV
//! conversion.

pub mod from_f06;
pub mod layout;
