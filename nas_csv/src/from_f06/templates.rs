//! This submodule implements conversion templates for the supported F06 block
//! types.

use std::collections::BTreeMap;

use f06::prelude::*;

use crate::prelude::*;
use crate::prelude::index_fns::*;

/// Macro to generate a sequence of ColumnValue generators.
macro_rules! cols {
  (
    $(
      $vnt:ident,
      [
        $(
          $before:expr,
        )*
      ],
      [
        $(
          $direct:expr,
        )*
      ],
      [
        $(
          $byvar:ident,
        )*
      ],
      [
        $(
          $after:expr,
        )*
      ],
    )*
  ) => {
    [
      $(
        $($before,)*
        $(
          ColumnGenerator::ColumnValue(NasIndex::$vnt($direct)),
        )*
        $(
          ColumnGenerator::ColumnValue(NasIndex::$vnt($vnt::$byvar)),
        )*
        $($after,)*
      )*
    ]
  };
}

/// Same as cols, but for types with inners.
macro_rules! cols_inner {
  (
    $(
      $outer:ident,
      $inner:ident,
      [
        $(
          $before:expr,
        )*
      ],
      [
        $(
          $direct:expr,
        )*
      ],
      [
        $(
          $byvar:ident,
        )*
      ],
      [
        $(
          $after:expr,
        )*
      ],
    )*
  ) => {
    [
      $(
        $($before,)*
        $(
          ColumnGenerator::ColumnValue(NasIndex::$outer($outer($direct))),
        )*
        $(
          ColumnGenerator::ColumnValue(NasIndex::$outer($outer($inner::$byvar))),
        )*
        $($after,)*
      )*
    ]
  };
}

/// Generator that always produces a zero.
pub const ZERO: ColumnGenerator = ColumnGenerator::ConstantNumber(
  F06Number::Natural(0)
);

/// Generator that always produces a one.
pub const ONE: ColumnGenerator = ColumnGenerator::ConstantNumber(
  F06Number::Natural(1)
);

/// Contains all the block converters in this source file.
pub const ALL_CONVERTERS: &[BlockConverter] = &[
  // displacements
  CT_DISPLACEMENTS,
  // grid point force balance
  CT_GPFORCEBALANCE,
  // stresses
  CT_STRESSES_QUAD,
  CT_STRESSES_TRIA,
  CT_STRESSES_ROD,
  CT_STRESSES_BAR,
  CT_STRESSES_ELAS1,
  // strains
  CT_STRAINS_QUAD,
  CT_STRAINS_TRIA,
  CT_STRAINS_ROD,
  CT_STRAINS_BAR,
  CT_STRAINS_ELAS1,
  // forces
  CT_FORCES_QUAD,
  CT_FORCES_TRIA,
  CT_FORCES_ROD,
  CT_FORCES_BAR,
  CT_FORCES_ELAS1
];

/// Returns all the converters in this source file, coded per-type.
pub fn all_converters() -> BTreeMap<BlockType, BlockConverter> {
  return ALL_CONVERTERS.iter()
    .copied()
    .map(|c| (c.input_block_type, c))
    .collect();
}

/// Conversion template for displacements blocks.
pub const CT_DISPLACEMENTS: BlockConverter = BlockConverter {
  input_block_type: BlockType::Displacements,
  output_block_id: CsvBlockId::Displacements,
  generators: &[
    cols!(
      Dof,
      [
        ColumnGenerator::GridId,
        ColumnGenerator::Subcase,
      ],
      [DOF_TX, DOF_TY, DOF_TZ, DOF_RX, DOF_RY, DOF_RZ,],
      [],
      [ZERO, ONE,],
    )
  ],
  headers: &[
    ["GID", "Subcase", "Tx", "Ty", "Tz", "Rx", "Ry", "Rx", "Csys", "Ptype"]
  ]
};

/// Conversion template for grid point force balance blocks.
pub const CT_GPFORCEBALANCE: BlockConverter = BlockConverter {
  input_block_type: BlockType::GridPointForceBalance,
  output_block_id: CsvBlockId::GridPointForces,
  generators: &[
    cols!(
      Dof,
      [
        ColumnGenerator::GridId,
        ColumnGenerator::Subcase,
        ColumnGenerator::WithDefault(
          &ColumnGenerator::ElementId, &CsvField::Natural(0)
        ),
        ColumnGenerator::RowIndexFn(&(ixfn_fo as IndexFn)),
      ],
      [DOF_TX, DOF_TY, DOF_TZ, DOF_RX, DOF_RY, DOF_RZ,],
      [],
      [],
    )
  ],
  headers: &[
    ["GID", "Subcase", "EID", "TYPE", "Fx", "Fy", "Fz", "Mx", "My", "Mz"]
  ]
};

/// Conversion template for quad stresses.
pub const CT_STRESSES_QUAD: BlockConverter = BlockConverter {
  input_block_type: BlockType::QuadStresses,
  output_block_id: CsvBlockId::Stresses,
  generators: &[
    cols!(
      PlateStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ColumnGenerator::WithDefault(
          &ColumnGenerator::GridId, &CsvField::Natural(0)
        ),
      ],
      [],
      [FibreDistance, NormalX, NormalY,],
      [ZERO,],
      PlateStressField,
      [],
      [],
      [ShearXY,],
      [ZERO, ZERO,],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", "GID", "FibreDistance",
      "NormalX", "NormalY", BLANK, "ShearXY", BLANK, BLANK
    ]
  ]
};

/// Conversion template for tria stresses.
pub const CT_STRESSES_TRIA: BlockConverter = BlockConverter {
  input_block_type: BlockType::TriaStresses,
  output_block_id: CsvBlockId::Stresses,
  generators: CT_STRESSES_QUAD.generators,
  headers: CT_STRESSES_QUAD.headers
};

/// Conversion template for rod stresses.
pub const CT_STRESSES_ROD: BlockConverter = BlockConverter {
  input_block_type: BlockType::RodStresses,
  output_block_id: CsvBlockId::Stresses,
  generators: &[
    cols!(
      RodStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [Axial,],
      [ZERO, ZERO,],
      RodStressField,
      [],
      [],
      [Torsional,],
      [ZERO, ZERO,],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", BLANK, BLANK, "Axial",
      BLANK, BLANK, "Torsional", BLANK, BLANK
    ]
  ]
};

/// Header for bar stresses.
const BAR_STRESSES_HEADER: [&str; 10] = [
  "EID", "Subcase", "GID", "End", "Axial", "S1", "S2", "S3", "S4", BLANK
];

/// Conversion template for bar stresses.
pub const CT_STRESSES_BAR: BlockConverter = BlockConverter {
  input_block_type: BlockType::BarStresses,
  output_block_id: CsvBlockId::Stresses,
  generators: &[
    cols!(
      BarStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [
        BarStressField::Axial,
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 1 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 2 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 3 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 4 },
      ],
      [],
      [ZERO,],
    ),
    cols!(
      BarStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ONE,
      ],
      [
        BarStressField::Axial,
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 1 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 2 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 3 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 4 },
      ],
      [],
      [ZERO,],
    )
  ],
  headers: &[BAR_STRESSES_HEADER, BAR_STRESSES_HEADER]
};

/// Conversion template for ELAS1 stresses.
pub const CT_STRESSES_ELAS1: BlockConverter = BlockConverter {
  input_block_type: BlockType::Elas1Stresses,
  output_block_id: CsvBlockId::Stresses,
  generators: &[
    cols!(
      SingleStress,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [Stress,],
      [ZERO, ZERO, ZERO, ZERO, ZERO,],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", BLANK, BLANK, "Stress",
      BLANK, BLANK, BLANK, BLANK, BLANK
    ]
  ]
};

/// Conversion template for quad stresses.
pub const CT_STRAINS_QUAD: BlockConverter = BlockConverter {
  input_block_type: BlockType::QuadStrains,
  output_block_id: CsvBlockId::Strains,
  generators: &[
    cols_inner!(
      PlateStrainField,
      PlateStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ColumnGenerator::WithDefault(
          &ColumnGenerator::GridId, &CsvField::Natural(0)
        ),
      ],
      [],
      [FibreDistance, NormalX, NormalY,],
      [ZERO,],
      PlateStrainField,
      PlateStressField,
      [],
      [],
      [ShearXY,],
      [ZERO, ZERO,],
    )
  ],
  headers: CT_STRESSES_QUAD.headers
};

/// Conversion template for tria stresses.
pub const CT_STRAINS_TRIA: BlockConverter = BlockConverter {
  input_block_type: BlockType::TriaStrains,
  output_block_id: CsvBlockId::Strains,
  generators: CT_STRAINS_QUAD.generators,
  headers: CT_STRESSES_TRIA.headers
};

/// Conversion template for rod stresses.
pub const CT_STRAINS_ROD: BlockConverter = BlockConverter {
  input_block_type: BlockType::RodStrains,
  output_block_id: CsvBlockId::Strains,
  generators: &[
    cols_inner!(
      RodStrainField,
      RodStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [Axial,],
      [ZERO, ZERO,],
      RodStrainField,
      RodStressField,
      [],
      [],
      [Torsional,],
      [ZERO, ZERO,],
    )
  ],
  headers: CT_STRESSES_ROD.headers
};

/// Conversion template for bar stresses.
pub const CT_STRAINS_BAR: BlockConverter = BlockConverter {
  input_block_type: BlockType::BarStrains,
  output_block_id: CsvBlockId::Strains,
  generators: &[
    cols_inner!(
      BarStrainField,
      BarStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [
        BarStressField::Axial,
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 1 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 2 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 3 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndA,  point: 4 },
      ],
      [],
      [ZERO,],
    ),
    cols_inner!(
      BarStrainField,
      BarStressField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ONE,
      ],
      [
        BarStressField::Axial,
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 1 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 2 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 3 },
        BarStressField::AtRecoveryPoint { end: BarEnd::EndB,  point: 4 },
      ],
      [],
      [ZERO,],
    )
  ],
  headers: CT_STRESSES_BAR.headers
};

/// Conversion template for ELAS1 stresses.
pub const CT_STRAINS_ELAS1: BlockConverter = BlockConverter {
  input_block_type: BlockType::Elas1Strains,
  output_block_id: CsvBlockId::Strains,
  generators: &[
    cols!(
      SingleStrain,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [Strain,],
      [ZERO, ZERO, ZERO, ZERO, ZERO,],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", BLANK, BLANK, "Strain",
      BLANK, BLANK, BLANK, BLANK, BLANK
    ]
  ]
};

/// Conversion template for quad forces.
pub const CT_FORCES_QUAD: BlockConverter = BlockConverter {
  input_block_type: BlockType::QuadForces,
  output_block_id: CsvBlockId::Forces,
  generators: &[
    cols!(
      PlateForceField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [NormalX, NormalY, NormalXY, MomentX, MomentY, MomentXY,],
      [],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", BLANK, BLANK, "NormalX",
      "NormalY", "NormalXY", "MomentX", "MomentY", "MomentXY"
    ]
  ]
};

/// Conversion template for tria forces.
pub const CT_FORCES_TRIA: BlockConverter = BlockConverter {
  input_block_type: BlockType::TriaForces,
  output_block_id: CsvBlockId::Forces,
  generators: CT_FORCES_QUAD.generators,
  headers: CT_FORCES_QUAD.headers
};

/// Conversion template for rod forces.
pub const CT_FORCES_ROD: BlockConverter = BlockConverter {
  input_block_type: BlockType::RodForces,
  output_block_id: CsvBlockId::Forces,
  generators: &[
    cols!(
      RodForceField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [AxialForce,],
      [ZERO, ZERO, ZERO, ZERO,],
      RodForceField,
      [],
      [],
      [Torque,],
      [],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", BLANK, BLANK, "Axial",
      BLANK, BLANK, BLANK, BLANK, "Torque"
    ]
  ]
};

/// Header for bar forces. It appears twice.
const BAR_FORCES_HEADER: [&str; 10] = [
  "EID", "Subcase", "GID", "End", "Axial", "S1", "S2", "M1", "M2", "Torque"
];

/// Conversion template for bar forces.
pub const CT_FORCES_BAR: BlockConverter = BlockConverter {
  input_block_type: BlockType::BarForces,
  output_block_id: CsvBlockId::Forces,
  generators: &[
    cols!(
      BarForceField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [
        BarForceField::AxialForce,
        BarForceField::Shear { plane: BarPlane::Plane1 },
        BarForceField::Shear { plane: BarPlane::Plane2 },
        BarForceField::BendMoment {end: BarEnd::EndA, plane: BarPlane::Plane1},
        BarForceField::BendMoment {end: BarEnd::EndA, plane: BarPlane::Plane2},
      ],
      [Torque,],
      [],
    ),
    cols!(
      BarForceField,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ONE,
      ],
      [
        BarForceField::AxialForce,
        BarForceField::Shear { plane: BarPlane::Plane1 },
        BarForceField::Shear { plane: BarPlane::Plane2 },
        BarForceField::BendMoment {end: BarEnd::EndB, plane: BarPlane::Plane1},
        BarForceField::BendMoment {end: BarEnd::EndB, plane: BarPlane::Plane2},
      ],
      [Torque,],
      [],
    )
  ],
  headers: &[BAR_FORCES_HEADER, BAR_FORCES_HEADER]
};

/// Conversion template for ELAS1 forces.
pub const CT_FORCES_ELAS1: BlockConverter = BlockConverter {
  input_block_type: BlockType::Elas1Forces,
  output_block_id: CsvBlockId::Forces,
  generators: &[
    cols!(
      SingleForce,
      [
        ColumnGenerator::ElementId,
        ColumnGenerator::Subcase,
        ZERO,
        ZERO,
      ],
      [],
      [Force,],
      [ZERO, ZERO, ZERO, ZERO, ZERO,],
    )
  ],
  headers: &[
    [
      "EID", "Subcase", BLANK, BLANK, "Force",
      BLANK, BLANK, BLANK, BLANK, BLANK
    ]
  ]
};
