# Manual for `f06csv`

###### by Bruno Borges Paschoalinoto, for version `0.3.5` (2024-01-28)

This document gives a thorough outline of the capabilities of the `f06csv`
utility.


## 1. Basic information about `f06csv`

### 1.1. What is it?

Put simply, it's meant to convert from the F06 format, the human-readable text
output produced by Nastran-like FEM solvers (e.g. NX/Simcenter Nastran,
MYSTRAN, Autodesk Inventor Nastran, MSC Nastran, NASTRAN-95, etc.) into a
well-organised, fixed-with table format -- a CSV (comma-separated values) file.

Discussed use cases for such a tool include but are not limited to:

  - Loading output data into spreadsheets for analysis (manual or automated),
  - Comparing data produced by different solvers by translating results into
  a common format, and
  - Future-proofing against dependency on software needed to work with
  proprietary and/or closed formats.

It's a work-in-progress, and part of a larger project.

### 1.2. Who made it and what for?

Not only `f06csv`, but this entire repository has been motivated, guided and funded by the [MYSTRAN](https://mystran.com) project. The primary author is
[Bruno Borges Paschoalinoto](https://bor.gs/en).

One of our aims having a F06, being a format that all Nastran-like solvers are known to produce *and*
also human-readable, was a 
