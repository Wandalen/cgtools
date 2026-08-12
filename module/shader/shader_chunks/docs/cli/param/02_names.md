# Parameter :: 2. names

- **Fundamental Type:** [`ChunkName`](../type/01_chunk_name.md) list
  (`Vec<String>` at the `src/lib.rs` boundary; unilang `Kind::String` with
  `ArgumentAttributes { multiple: true, .. }`)
- **Constraints:** Every element must resolve via
  `shader_chunks_core::chunk_get` ( an O(1) lookup against the
  `shader_chunks_core::CHUNKS` table ) — the first
  unresolved element returns `CliError::UnknownChunk`; a resolved-but-
  incomplete set (e.g. a chunk without its declared dependency) returns
  `CliError::Compose(ComposeError::MissingDependency)` from
  `shader_chunks_core::try_compose`, not a panic
- **Default:** **(required)** — `compose` needs at least one chunk to
  produce any output; there is no meaningful "compose nothing" default
- **Purpose:** The set of chunks to compose into one WGSL preview, in any
  input order — `shader_chunks_core::try_compose` performs dependency-ordered
  topological composition internally, so `compose value_noise hash21` and
  `compose hash21 value_noise` produce identical output.

### Examples
```bash
# Valid values
names::hash21                        # single chunk, no dependencies
names::hash21 names::value_noise     # order-independent; unilang collects
                                      # repeated `names::` occurrences (or a
                                      # single positional multi-token run)
                                      # into one Value::List

# Invalid values (rejected with error)
names::bogus_chunk    # "unknown chunk: `bogus_chunk` (see `list` for valid names)"
names::value_noise    # resolves, but omits its `hash21` dependency:
                       # ComposeError::MissingDependency, non-zero exit
```

### Notes
- Positional, multiple — `compose hash21 value_noise` is the primary
  invocation form; `unilang`'s `multiple: true` attribute collects every
  trailing positional token into a single `Value::List`, mirrored in
  `src/main.rs`'s `cmd_compose` routine, which also accepts a bare
  `Value::String` (a single chunk with no list wrapping) as the one-element
  case.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.compose](../command/05_compose.md) | **(required)** | At least one chunk name; dependency order is resolved automatically |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [ChunkName](../type/01_chunk_name.md) | String (list) | `Vec<String>` | Every element must resolve in `shader_chunks_core::CHUNKS` |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
