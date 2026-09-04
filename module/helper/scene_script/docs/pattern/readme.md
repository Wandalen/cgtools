# Pattern Doc Definition

A **pattern** documents a reusable solution to a recurring design problem, its applicability, and its trade-offs. In `scene_script`, this collection is the navigational hub for solutions specific to registering foreign types into an embedded Rhai engine — distinct from the root-level `docs/pattern/` collection, which documents the two script *forms* (data vs. glue) this crate serves. This collection holds one instance per reusable solution; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s reusable binding-registration solutions.
- **Responsibility**: Document each pattern's problem, solution, applicability, and consequences.
- **In Scope**: Techniques for exposing foreign Rust types/functions to Rhai scripts.
- **Out of Scope**: The declarative/imperative script-form patterns these bindings serve (see root [`docs/pattern/004`](../../../../../docs/pattern/004_script_as_data.md), [`005`](../../../../../docs/pattern/005_script_as_glue.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Manual CustomType Registration for Foreign Types](001_manual_customtype_registration_for_foreign_types.md) | Register a foreign type into Rhai by hand, sidestepping the orphan rule | ✅ |
| 002 | [Dual-Precision Side-by-Side Registration](002_dual_precision_side_by_side_registration.md) | Extend the script surface with a new precision or arity without breaking existing names | ✅ |
