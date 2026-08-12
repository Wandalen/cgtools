# Type :: 1. ChunkName

**Purpose:** Identifies one shader chunk in `shader_chunks::ALL_CHUNKS` by
the value of its `//@ name: <value>` metadata comment — the identifier every
command that targets a specific chunk (`get`, `tree`, `compose`) accepts.

**Fundamental Type:** `String` (unilang `Kind::String`). No dedicated Rust
wrapper struct exists — `shader_chunks_cli` is one small, read-only
inspection CLI with exactly one string-shaped identifying concept, so the
validation this type performs is realized as a runtime lookup
(`find_chunk` in `src/lib.rs`) rather than a compile-time-distinct newtype.
Documented here as a semantic type because it carries real constraints and
a real parse/validate step — not because a `ChunkName` struct exists in the
source.

**Constraints:**
- Must exactly match the `name` a bundled chunk's leading `//@ name: <value>`
  WGSL comment parses to, via `shader_chunks::parse_name`
- Case-sensitive, no normalization or fuzzy matching
- The valid set is closed and enumerable: run `list` to see every accepted
  value (currently `hash21`, `value_noise`, `fbm3`, `fullscreen_triangle`)

**Parsing:** `find_chunk(name: &str)` (`src/lib.rs`) linearly scans
`shader_chunks::ALL_CHUNKS`, calling `shader_chunks::parse_name` on each
bundled WGSL blob and comparing for an exact string match. No match →
`CliError::UnknownChunk(name)`, reported to the user as `` unknown chunk:
`<name>` (see `list` for valid names) `` on stderr with a non-zero exit —
never a panic.

**Methods:**
- `get() -> &str` — the raw name string is the value itself; no separate
  accessor exists, matching there being no wrapper struct (see Fundamental
  Type above)
- `is_known(name) -> bool` — conceptually, `find_chunk(name).is_ok()`;
  realized as the `Result` returned by every command function that
  resolves a name (`get_chunk`, `tree_chunk`, `compose_chunks`), not as a
  standalone boolean-returning method

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.get`](../command/02_get.md) | `name` |
| 2 | [`.tree`](../command/04_tree.md) | `name` |
| 3 | [`.compose`](../command/05_compose.md) | `names` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`name`](../param/01_name.md) | 2 |
| 2 | [`names`](../param/02_names.md) | 1 |
