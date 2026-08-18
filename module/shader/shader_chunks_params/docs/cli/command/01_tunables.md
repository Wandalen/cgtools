# Command :: 6. tunables

### Description

Lists every tunable parameter `shader_chunks_params::chunk_discover` finds
declared on one bundled chunk's `//@ param:` lines — name, kind, WGSL
type, range, and range source (declared vs. inferred). A chunk that
declares none (a handful of leaf/infrastructure chunks — see Notes)
prints an explicit "no tunable parameters" message instead of a blank
table or an error.

-- **Parameters:** name
-- **Exit Codes:** 0 (success, including the zero-parameters case) | 1
   (`name` does not resolve against `shader_chunks_core::CHUNKS`)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks tunables <name>
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../../../../shader_chunks_query/docs/cli/param/01_name.md) | — | Yes | The bundled chunk whose declared tunable parameters to list |

### Examples
```bash
shader_chunks tunables hash21
# chunk `hash21` declares no tunable parameters

shader_chunks tunables fbm3
# name        kind      type  range  source
# ----------  --------  ----  -----  --------
# lacunarity  Argument  F32   1..3   Declared
# gain        Argument  F32   0..1   Declared
```

Most bundled chunks declare one or more `//@ param:` lines today (see
Notes); `fbm3` above is a real, subprocess-reachable example of the
populated-table path — one table row per parameter, columns `name`,
`kind`, `type`, `range`, `source`. A handful of leaf/infrastructure
chunks — `hash21` among them — still declare none and render the
explicit empty message instead. See
[`shader_chunks_params`](../../../../shader_chunks_params/readme.md) for
the `//@ param:` declaration grammar and range-inference heuristic, and
`shader_chunks_params/tests/tunables_test.rs::tunables_of_chunk_lists_declared_and_inferred_parameters`
for a fixture-based worked example covering both a declared and an
inferred range in the same table.

### Notes
- 45 of the 50 bundled chunks declare one or more `//@ param:` lines
  today; `hash21`, `hash22`, `palette_cosine`, `srgb`, and
  `fullscreen_triangle` are the remaining leaf/infrastructure chunks that
  still declare none and so still render the empty-message path — both
  paths are real, subprocess-reachable, tested outcomes, not a gap.
- The empty-message case is a single explanatory line, not a table —
  distinct from a hypothetical zero-row table.
- `range`/`source` render `-`/`-` for a parameter with no numeric range
  (a `texture`-kind parameter, or a `bool`-typed one) — see
  `shader_chunks_params::range_infer`.
- Output format: [`table_plain`](../../../../shader_chunks_query/docs/cli/format/01_table_plain.md) (same
  rendering pipeline as `list`/`get`/`tags`) when at least one parameter
  is declared.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.get`](../../../../shader_chunks_query/docs/cli/command/02_get.md) | Flat metadata about a chunk, not its tunable parameters |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
