# Invariant Doc Entity

### Scope

- **Purpose**: Navigational hub for correctness properties that must always hold in `tiles_tools`.
- **Responsibility**: Document each invariant's statement, enforcement mechanism, and violation consequences.
- **In Scope**: Triangular coordinate sum constraint.
- **Out of Scope**: The algorithms that rely on this invariant (see `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Triangular Coordinate Sum Constraint](001_triangular_coordinate_sum_constraint.md) | Why `a + b + c` must stay within `{0, ±1}` for `triangular::Coordinate` | ✅ |
