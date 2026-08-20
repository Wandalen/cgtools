# Invariant Tests

### Scope

- **Purpose:** Test specifications proving each of this crate's correctness
  properties is enforced, not merely documented.
- **Responsibility:** One file per invariant, documenting the observable
  guarantee independent of any single call site.
- **In Scope:** Invariant 001 (Top-Level Bindings Convention) and invariant
  004 (Script-As-Data Purity) — the 2 elements touched by
  `task/verified/416_scene_script_production_lint_enforcement.md`'s declared
  scope (work-item-driven expansion — see `l1_imp_surface.rulebook.md §
  Expansion : Procedure - Expand Test Surface`), not a full sweep of all 4
  invariants in [`../../../docs/invariant/`](../../../docs/invariant/readme.md).
- **Out of Scope:** Invariant 002 (F32x2/F64x2 Type Distinctness) and
  invariant 003 (Rhai-Facing Names Mirror Rust Identifiers) — not touched by
  any work item scoping an expansion pass yet.

---

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| [01_top_level_bindings_convention.md](01_top_level_bindings_convention.md) | `invariant` spec for Top-Level Bindings Convention | ⏳ |
| [04_script_as_data_purity.md](04_script_as_data_purity.md) | `invariant` spec for Script-As-Data Purity | ⏳ |

**Total:** 2 invariant test specs (of 4 defined in
[`../../../docs/invariant/`](../../../docs/invariant/readme.md))

### Docs

| File | Relationship |
|------|----------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/invariant/readme.md`](../../../docs/invariant/readme.md) | Invariant documentation source |
