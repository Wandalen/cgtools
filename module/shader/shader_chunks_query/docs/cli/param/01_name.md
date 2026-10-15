# Parameter :: 1. name

- **Fundamental Type:** [`ChunkName`](../type/01_chunk_name.md) (`String` wrapped
  conceptually; unilang `Kind::String`)
- **Constraints:** Must resolve via `shader_chunks_core::chunk_get` — an O(1)
  lookup against the `shader_chunks_core::CHUNKS` table; an unresolved name
  returns `CliError::UnknownChunk`, never a panic
- **Default:** `Varies` — optional on `tree` (absent means "every chunk
  nothing else depends on", the full forest, not an error); required on
  `tunables` (no omission form); optional on `preview`/`render` in a
  third sense — absent means "the target is `file::` instead," never a
  bare omission (exactly one of `name`/`file::` is required there)
- **Purpose:** Identifies the single bundled chunk a command operates on —
  the root of `tree`'s dependency rendering, the chunk whose declared
  tunable parameters `tunables` lists, the chunk `preview` builds a
  live browser bundle from, or the chunk `render` freezes to a static
  PNG. Optional on `tree` because "show me everything" (the forest view)
  is itself a meaningful, common request — not a missing-argument error.
  Optional on `preview`/`render` for a different reason — it names one
  of two mutually exclusive targets, not a standalone optional filter.

### Examples
```bash
# Valid values
name::hash21        # a real bundled chunk (positional form: `tree hash21`)
name::fbm3

# Invalid values (rejected with error)
name::bogus_chunk    # "unknown chunk: `bogus_chunk` (see `list` for valid names)"
```

### Notes
- Positional, not `key::value` — `shader_chunks` follows unilang's
  positional-argument convention for a command's sole identifying argument
  (`tree fbm3` / `tunables fbm3`, not `tree name::fbm3`); unilang resolves
  a positional token against the declared `ArgumentDefinition` by
  position, so either form is accepted.
- `tree`, `tunables`, `preview`, and `render` are the only commands
  taking a *single* name — `get` moved to the plural
  [`names`](02_names.md) when it adopted the shared query engine
  (multiple chunks per invocation are meaningful there).
- On `preview` and `render`, giving both `name` and `file::`, or
  neither, fails with exit 1 before either is resolved — see
  [`file`](../../../../shader_chunks_preview/docs/cli/param/01_file.md)'s own Notes for the exact error text.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.tree](../command/04_tree.md) | `Varies` (omit for the full forest) | Absent means "show every root chunk," not an error |
| 2 | [.tunables](../../../../shader_chunks_params/docs/cli/command/01_tunables.md) | — (required) | No omission form — `tunables` always names exactly one chunk |
| 3 | [.preview](../../../../shader_chunks_preview/docs/cli/command/01_preview.md) | — (required unless `file::` given) | Mutually exclusive with `file::`; omit when passing `file::` |
| 4 | [.render](../../../../shader_chunks_render/docs/cli/command/01_render.md) | — (required unless `file::` given) | Same exclusivity; names the chunk to freeze as a PNG |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [ChunkName](../type/01_chunk_name.md) | String | `String` | Must resolve in `shader_chunks_core::CHUNKS` |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
