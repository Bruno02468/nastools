//! This submodule implements conversion templates for the supported F06 block
//! types.

use f06::prelude::*;

use crate::prelude::*;

/// Conversion template for the displacements block.
pub const CT_DISPLACEMENTS: BlockConverter = BlockConverter {
  input_block_type: BlockType::Displacements,
  output_block_id: CsvBlockId::Displacements,
  generators: &[
    [
      ColumnGenerator::GridId,
      ColumnGenerator::Subcase,
      ColumnGenerator::ColumnValue(NasIndex::Dof(DOF_TX)),
      ColumnGenerator::ColumnValue(NasIndex::Dof(DOF_TY)),
      ColumnGenerator::ColumnValue(NasIndex::Dof(DOF_TZ)),
      ColumnGenerator::ColumnValue(NasIndex::Dof(DOF_RX)),
      ColumnGenerator::ColumnValue(NasIndex::Dof(DOF_RY)),
      ColumnGenerator::ColumnValue(NasIndex::Dof(DOF_RZ)),
      ColumnGenerator::ConstantNumber(F06Number::Natural(0)),
      ColumnGenerator::ConstantNumber(F06Number::Natural(1))
    ]
  ],
};
