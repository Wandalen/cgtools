# Parameters

### Scope

- **Purpose:** Documents every parameter owned by this crate.
- **Responsibility:** One dedicated file per parameter.
- **In Scope:** The 1 parameter this crate introduces — `out`.
- **Out of Scope:** Command-level syntax/examples (→ [`../command/`](../command/readme.md)),
  the `names`/`transitive` parameters `compose` also accepts, owned by
  [`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/param/readme.md),
  and the family's other parameters generally (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Parameter | Type | Default | Status |
|---|------|-----------|------|---------|--------|
| 1 | [01_out.md](01_out.md) | `out` | String | None (prints to stdout) | ✅ |

**Total:** 1 parameter owned by this crate (of 32 across the
`shader_chunks` family; `compose` accepts 3 total — this 1 plus the
shared `names`/`transitive` pair).

**Co-occurrence note:** `out` belongs to no
[parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) —
an artifact-path selector, not a filter/projection/formatting modifier,
same as `render`'s own `out`. Unlike every other parameter default in
the CLI, `out`'s absence is not "use a computed value" but "use a
different destination entirely" (stdout instead of a file).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Sole command accepting this parameter |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/param/readme.md](../../../tests/docs/cli/param/readme.md) | Parameter-level test specifications |
