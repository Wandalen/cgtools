# Data Structure Doc Definition

A **data structure** documents a data container's shape, membership invariants, and access operations — a generic structural pattern, not a domain-typed value with business semantics (contrast: `type/`). In `scene_script`, this collection is the navigational hub for `Tween`'s script-facing shape: a projection of a foreign, generic-over-vector-type Rust container into Rhai's type system, distinct from `animation`'s own (out-of-scope) documentation of it. This collection holds one instance per distinct script-facing shape; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for the shape of generic container types a Rhai script can hold in a variable.
- **Responsibility**: Document each registered container type's script-visible fields, mutability, and construction.
- **In Scope**: `Tween` exactly as Rhai's type registry exposes it — one script-visible name regardless of which vector type backs a given instance.
- **Out of Scope**: `animation::Tween<T>`'s actual Rust definition and full API — owned by `animation`, not re-documented here; `F32x2`/`F64x2`, which are domain-typed vector values rather than generic containers (see [`type/`](../type/readme.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Tween Script-Facing Type](001_tween_script_facing_type.md) | An opaque interpolation handle with no readable fields | ✅ |
