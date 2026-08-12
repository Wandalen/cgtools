# Parameter :: 1. name

- **Fundamental Type:** [`ChunkName`](../type/01_chunk_name.md) (`String` wrapped
  conceptually; unilang `Kind::String`)
- **Constraints:** Must resolve via `shader_chunks_core::chunk_get` — an O(1)
  lookup against the `shader_chunks_core::CHUNKS` table; an unresolved name
  returns `CliError::UnknownChunk`, never a panic
- **Default:** **(required)** for `get` — no default, since detail output is
  meaningless without a target chunk; `Varies` for `tree` — absent means
  "every chunk nothing else depends on" (the full forest), not an error
- **Purpose:** Identifies which single bundled chunk a command should act
  on. `get` treats it as mandatory. `tree` treats it as optional because
  "show me everything" (the forest view) is itself a meaningful, common
  request — not a missing-argument error.

### Examples
```bash
# Valid values
name::hash21        # a real bundled chunk (positional form: `get hash21`)
name::fbm3

# Invalid values (rejected with error)
name::bogus_chunk    # "unknown chunk: `bogus_chunk` (see `list` for valid names)"
```

### Notes
- Positional, not `key::value` — `shader_chunks` follows unilang's
  positional-argument convention for a command's sole identifying argument
  (`get hash21`, not `get name::hash21`); unilang resolves a positional
  token against the declared `ArgumentDefinition` by position, so either
  form is accepted.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.get](../command/02_get.md) | **(required)** | Detail output requires exactly one target chunk |
| 2 | [.tree](../command/04_tree.md) | `Varies` (omit for the full forest) | Absent means "show every root chunk," not an error |

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
