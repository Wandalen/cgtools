# Parameters

### Scope

- **Purpose:** Documents every parameter this crate introduces.
- **Responsibility:** One dedicated file per parameter, unified across
  every command that accepts it.
- **In Scope:** The 2 parameters `preview` introduces — `file`, `serve`.
- **Out of Scope:** `preview`'s third parameter, `name` — owned by
  [`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/param/01_name.md)
  since the majority of the family's commands accept it; command-level
  syntax/examples (→ [`../command/`](../command/readme.md)); the
  remaining parameters of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Parameter | Type | Default | Status |
|---|------|-----------|------|---------|--------|
| 1 | [01_file.md](01_file.md) | `file` | String | off (mutually exclusive alternative to `name`) | ✅ |
| 2 | [02_serve.md](02_serve.md) | `serve` | [`Switch`](../../../../shader_chunks_query/docs/cli/type/07_switch.md) | `true` | ✅ |

**Total:** 2 own parameters (of 26 across the `shader_chunks` family)

**Group membership:** neither parameter belongs to a
[parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md)
— `file` is a target selector (mutually exclusive with `name`, not a
filter/projection/format modifier) and `serve` is a side-effect toggle
(browser hand-off). Both are introduced by, and scoped to, `.preview`.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Commands accepting these parameters |
| [`name`](../../../../shader_chunks_query/docs/cli/param/01_name.md) | Sibling target selector, owned by `shader_chunks_query` |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/param/readme.md](../../../tests/docs/cli/param/readme.md) | Parameter-level test specifications |
