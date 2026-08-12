# Type Doc Definition

A **type** instance documents one significant Domain Type's shape, domain meaning, and construction/validation rules — value objects, DDD entities, or DTOs whose business semantics matter beyond their code definition. In `scene_script`, this collection holds the script-visible vector value types: each gets its own instance so the script's registered type vocabulary maps one-to-one with a doc instance, rather than one instance bundling both precisions. This collection holds one instance per Domain Type; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s script-visible Domain Types.
- **Responsibility**: Document each type's domain meaning, its `domain`/`ddd` classification, and its construction/validation rules.
- **In Scope**: `F32x2` and `F64x2` as Rhai-registered value objects — each is its own instance.
- **Out of Scope**: Generic containers with no independent domain meaning of their own (see `data_structure/` — e.g. `Tween`'s opaque handle, a mechanism generic over vector type, not itself a domain value); full call signatures and error behavior (see `api/001`); the foreign Rust structs these types project into Rhai (see `pattern/001`).

### Overview Table

| ID | Name | domain | ddd | Status |
|----|------|--------|-----|--------|
| 001 | [F32x2 (Script-Facing Vector Value)](001_f32x2_script_facing_vector_value.md) | vector | value_object | ✅ |
| 002 | [F64x2 (Script-Facing Vector Value)](002_f64x2_script_facing_vector_value.md) | vector | value_object | ✅ |
