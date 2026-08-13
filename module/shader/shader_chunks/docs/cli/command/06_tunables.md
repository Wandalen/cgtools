# Command :: 6. tunables

### Description

Lists every tunable parameter `shader_chunks_params::discover_chunk` finds
declared on one bundled chunk's `//@ param:` lines — name, kind, WGSL
type, range, and range source (declared vs. inferred). A chunk that
declares none (true for all 4 bundled chunks today — see Notes) prints an
explicit "no tunable parameters" message instead of a blank table or an
error.

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
| `name` | [`ChunkName`](../param/01_name.md) | — | Yes | The bundled chunk whose declared tunable parameters to list |

### Examples
```bash
shader_chunks tunables hash21
# chunk `hash21` declares no tunable parameters

shader_chunks tunables fbm3
# chunk `fbm3` declares no tunable parameters
```

Every bundled chunk today declares zero `//@ param:` lines, so both real
invocations above render the explicit empty message (see Notes). When a
chunk does declare one or more, `tunables` instead renders one table row
per parameter — columns `name`, `kind`, `type`, `range`, `source` — see
[`shader_chunks_params`](../../../../shader_chunks_params/readme.md) for
the `//@ param:` declaration grammar and range-inference heuristic, and
`shader_chunks_test.rs::tunables_of_chunk_lists_declared_and_inferred_parameters`
for a worked example against a fixture chunk.

### Notes
- Every bundled chunk today declares zero `//@ param:` lines — annotating
  one is out of scope for this command, same as it was for
  `shader_chunks_params` itself (decision Q-03); the empty-message path
  is the only outcome reachable against the real bundled set via
  subprocess, and is itself a tested, valid outcome, not a gap.
- The empty-message case is a single explanatory line, not a table —
  distinct from a hypothetical zero-row table.
- `range`/`source` render `-`/`-` for a parameter with no numeric range
  (a `texture`-kind parameter, or a `bool`-typed one) — see
  `shader_chunks_params::infer_range`.
- Output format: [`table_plain`](../format/01_table_plain.md) (same
  rendering pipeline as `list`/`get`/`tags`) when at least one parameter
  is declared.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.get`](02_get.md) | Flat metadata about a chunk, not its tunable parameters |

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
