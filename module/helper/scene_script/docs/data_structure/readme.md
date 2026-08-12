# Data Structure Doc Definition

A **data structure** documents a data container's shape, membership invariants, and access operations. In `scene_script`, this collection is the navigational hub for the script-facing shape of the types registered into Rhai — a projection of foreign Rust types into Rhai's type system, distinct from the originating crates' own (out-of-scope) data structure documentation. This collection holds one instance per distinct script-facing shape; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for the shape of the types a Rhai script can hold in a variable.
- **Responsibility**: Document each registered type's script-visible fields, mutability, and construction.
- **In Scope**: `F32x2`/`F64x2`/`Tween` exactly as Rhai's type registry exposes them.
- **Out of Scope**: `ndarray_cg::F32x2`/`F64x2` and `animation::Tween<T>`'s actual Rust definitions and full APIs — owned by those crates, not re-documented here.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [F32x2 / F64x2 Script-Facing Vector Types](001_f32x2_f64x2_script_facing_vector_types.md) | Two precision variants of a 2-component, get-only vector value | ✅ |
| 002 | [Tween Script-Facing Type](002_tween_script_facing_type.md) | An opaque interpolation handle with no readable fields | ✅ |
