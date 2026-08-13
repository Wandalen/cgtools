# Type :: 1. ChunkName

**Purpose:** Identifies one shader chunk in `shader_chunks_core::CHUNKS` by
the value of its `//@ name: <value>` metadata comment — the identifier every
command that targets specific chunks (`list`, `get`, `tree`, `compose`,
`tunables`) accepts, whether as a positional selector (`names`/`name`) or
as the `depends_on::` filter value.

**Fundamental Type:** `String` (unilang `Kind::String`). No dedicated Rust
wrapper struct exists — `shader_chunks` is one small, read-only
inspection CLI with exactly one string-shaped identifying concept, so the
validation this type performs is realized as a runtime lookup
(`find_chunk` in `src/lib.rs`) rather than a compile-time-distinct newtype.
Documented here as a semantic type because it carries real constraints and
a real parse/validate step — not because a `ChunkName` struct exists in the
source.

**Constraints:**
- Must exactly match a `shader_chunks_core::CHUNKS` row's `name` field — which
  mirrors that chunk's leading `//@ name: <value>` WGSL comment
- Case-sensitive, no normalization or fuzzy matching
- The valid set is closed and enumerable: run `list` to see every accepted
  value (currently `hash21`, `value_noise`, `fbm3`, `fullscreen_triangle`)

**Parsing:** `find_chunk(name: &str)` (`src/lib.rs`) resolves the name via
`shader_chunks_core::chunk_get` — an O(1) lookup into the
`shader_chunks_core::CHUNKS` descriptor table, no scan, no manifest
parsing. No match →
`CliError::UnknownChunk(name)`, reported to the user as `` unknown chunk:
`<name>` (see `list` for valid names) `` on stderr with a non-zero exit —
never a panic.

**Methods:**
- `get() -> &str` — the raw name string is the value itself; no separate
  accessor exists, matching there being no wrapper struct (see Fundamental
  Type above)
- `is_known(name) -> bool` — conceptually, `find_chunk(name).is_ok()`;
  realized as the `Result` returned by every routine that resolves a name
  (`query_chunks`, `tree_chunk`, `compose_chunks`, `tunables`), not as a
  standalone boolean-returning method

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `names` (optional), `depends_on::` |
| 2 | [`.get`](../command/02_get.md) | `names`, `depends_on::` |
| 3 | [`.tree`](../command/04_tree.md) | `name` |
| 4 | [`.compose`](../command/05_compose.md) | `names` |
| 5 | [`.tunables`](../command/06_tunables.md) | `name` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`name`](../param/01_name.md) | 2 |
| 2 | [`names`](../param/02_names.md) | 3 |
| 3 | [`depends_on`](../param/08_depends_on.md) | 2 |
