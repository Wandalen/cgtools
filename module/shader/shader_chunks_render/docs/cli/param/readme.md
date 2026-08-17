# Parameters

### Scope

- **Purpose:** Documents every parameter owned by this crate.
- **Responsibility:** One dedicated file per parameter.
- **In Scope:** The 4 parameters this crate introduces —
  `out`, `size`, `time`, `set`.
- **Out of Scope:** Command-level syntax/examples (→ [`../command/`](../command/readme.md)),
  type constraints/parsing (→ [`../type/`](../type/readme.md)), the 2
  target-selector parameters `render` also accepts —
  [`name`](../../../../shader_chunks_query/docs/cli/param/01_name.md)
  (owned by `shader_chunks_query`) and
  [`file`](../../../../shader_chunks_preview/docs/cli/param/01_file.md)
  (owned by `shader_chunks_preview`) — and the family's other 22
  parameters generally (→ [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Parameter | Type | Default | Status |
|---|------|-----------|------|---------|--------|
| 1 | [01_out.md](01_out.md) | `out` | String | `<target>.png` — derived from whichever target (`name`/`file`) was given | ✅ |
| 2 | [02_size.md](02_size.md) | `size` | String | `256` | ✅ |
| 3 | [03_time.md](03_time.md) | `time` | [`Float`](../type/01_float.md) | `0` | ✅ |
| 4 | [04_set.md](04_set.md) | `set` | [`ParameterOverride`](../type/02_parameter_override.md) list | none (bundle defaults) | ✅ |

**Total:** 4 parameters owned by this crate (of 28 across the
`shader_chunks` family; `render` accepts 6 total — these 4 plus the
shared `name`/`file` target pair).

**Co-occurrence note:** these 4 parameters belong to no
[parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) —
artifact-path/shape selectors, not filter/projection/formatting
modifiers. `out`'s default is the only parameter default in the entire
CLI that depends on another parameter's value (whichever of `name`/`file`
was given).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Sole command accepting these parameters |
| [../type/readme.md](../type/readme.md) | Type definitions |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/param/readme.md](../../../tests/docs/cli/param/readme.md) | Parameter-level test specifications |
