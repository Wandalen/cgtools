# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on. In `tiles_tools`, that covers correctness properties such as the triangular coordinate sum constraint and lattice address primacy, each written down with its enforcement mechanism and the consequences of violating it. This collection holds one instance per invariant, each pinned to where it is enforced in code; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for correctness properties that must always hold in `tiles_tools`.
- **Responsibility**: Document each invariant's statement, enforcement mechanism, and violation consequences.
- **In Scope**: Triangular coordinate sum constraint; lattice address primacy.
- **Out of Scope**: The algorithms that rely on these invariants (see `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Triangular Coordinate Sum Constraint](001_triangular_coordinate_sum_constraint.md) | Why `a + b + c` must stay within `{0, ±1}` for `triangular::Coordinate` | ✅ |
| 002 | [Lattice Address Primacy](002_lattice_address_primacy.md) | Spatial state lives at typed lattice coordinates; pixels are derived, never stored | ✅ |
