# Types

### Scope

- **Purpose:** Documents every semantic parameter type this crate
  introduces.
- **Responsibility:** One dedicated file per type — purpose, fundamental
  representation, constraints, parsing, methods.
- **In Scope:** The 2 types this crate introduces — `Float`, the CLI's
  only `Kind::Float`-typed parameter (`f32`, backing [`time`](../param/03_time.md)),
  and `ParameterOverride`, one `<property>:<value>` element of
  [`set`](../param/04_set.md)'s override list. `out`/`size` reuse the
  plain `String` fundamental (`size`'s two-form grammar is validated by
  this crate's own `size_parse`, not a dedicated type file).
- **Out of Scope:** Per-parameter defaults/requiredness (→ [`../param/`](../param/readme.md)),
  the family's other 10 types — owned by
  [`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/type/readme.md)
  (→ [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Type | Fundamental | Status |
|---|------|------|-------------|--------|
| 1 | [01_float.md](01_float.md) | Float | `f32` (via `Kind::Float`) | ✅ |
| 2 | [02_parameter_override.md](02_parameter_override.md) | ParameterOverride | `(String, f64)` (list element) | ✅ |

**Total:** 2 types owned by this crate (of 12 across the `shader_chunks`
family)

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../param/readme.md](../param/readme.md) | Parameter carrying this type |
| [../command/readme.md](../command/readme.md) | Command using this type via a parameter |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/type/readme.md](../../../tests/docs/cli/type/readme.md) | Type-level test specifications |
