# Parameter :: 8. depends_on

- **Fundamental Type:** [`ChunkName`](../type/01_chunk_name.md) (unilang
  `Kind::String`)
- **Constraints:** Must resolve via `shader_chunks_core::chunk_get` —
  an unknown chunk is `CliError::UnknownChunk`, non-zero exit (unlike
  [`stage::`](07_stage.md), this value names a registry row and is
  validated as one)
- **Default:** off (no dependency filtering)
- **Purpose:** Keeps only chunks that *directly* depend on the given
  chunk; [`transitive::1`](09_transitive.md) widens the walk to the full
  transitive closure.

### Examples
```bash
# Valid values
list depends_on::hash21 format::names                  # value_noise
list depends_on::hash21 transitive::1 format::names    # value_noise, fbm3

# Invalid values (rejected with error)
list depends_on::bogus   # "unknown chunk: `bogus` (see `list` for valid names)"
```

### Notes
- Direction: this selects *dependents of* the given chunk (who uses it),
  not its dependencies — for the latter, `tree <name>` renders the
  chunk's own dependency chain.
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | off | Narrows the full registry |
| 2 | [.get](../command/02_get.md) | off | Narrows the named selection |

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
