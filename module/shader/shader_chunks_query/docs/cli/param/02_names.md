# Parameter :: 2. names

- **Fundamental Type:** [`ChunkName`](../type/01_chunk_name.md) list
  (`Vec<String>` at each consuming crate's boundary — `QueryParams.names`
  in `shader_chunks_query_core/src/lib.rs` for `list`/`get`, the `compose`
  routine's own signature in `shader_chunks_compose/src/lib.rs`; unilang
  `Kind::List` with `ArgumentAttributes { multiple: true, .. }`)
- **Constraints:** Every element must resolve via
  `shader_chunks_core::chunk_get` ( an O(1) lookup against the
  `shader_chunks_core::CHUNKS` table ) — the first
  unresolved element returns `CliError::UnknownChunk`, never a panic. For
  `compose` only, a resolved-but-incomplete set (e.g. a chunk without its
  declared dependency) additionally returns
  `CliError::Compose(ComposeError::MissingDependency)` from
  `shader_chunks_core::try_compose`
- **Default:** `Varies` — **(required)** for `get` and `compose` (neither
  has a meaningful "no chunks" output); optional for `list`, where absent
  means "every chunk in the registry"
- **Purpose:** Selects the candidate chunk set an invocation operates on.
  For `list`/`get` it fixes the query engine's selection (in the given
  order, duplicates allowed) before [filtering](../param_group/01_filtering.md)
  applies; for `compose` it names the chunks to concatenate, in any input
  order — `shader_chunks_core::try_compose` resolves dependency order
  internally.

### Examples
```bash
# Valid values
get hash21                   # one chunk, expanded detail record
get hash21 fbm3              # two records, in the given order
list hash21 fbm3 hash21      # selection order kept, duplicates allowed
compose hash21 value_noise   # order-independent; dependency-ordered output

# Invalid values (rejected with error)
get bogus_chunk        # "unknown chunk: `bogus_chunk` (see `list` for valid names)"
compose value_noise    # resolves, but omits its `hash21` dependency:
                        # ComposeError::MissingDependency, non-zero exit
get                     # "The required argument 'names' is missing", non-zero exit
```

### Notes
- Positional, multiple — `get hash21 fbm3` is the primary invocation form;
  `unilang`'s `multiple: true` attribute collects every trailing positional
  token into a single `Value::List`. All three consuming routines flatten
  the nested list-of-lists unilang produces for positional multiples
  (`names_flatten` in `shader_chunks_cli_core/src/lib.rs`).
- Selection semantics differ from filter semantics: for `list`/`get`,
  `names` fixes *which chunks enter* the query in *which order*
  (`sort::input` preserves it) — the named-parameter filters then narrow
  that set. This is why `names` belongs to no
  [parameter group](../param_group/readme.md).
- The requiredness split is the single surface difference between `list`
  and `get` — see
  [`command_group/01_query.md`](../command_group/01_query.md).

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `Varies` (omit for every chunk) | Optional selection narrowing |
| 2 | [.get](../command/02_get.md) | **(required)** | Detail output needs ≥1 target chunk |
| 3 | [.compose](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | **(required)** | At least one chunk name; dependency order is resolved automatically |

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
