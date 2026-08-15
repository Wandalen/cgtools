# Formats

### Scope

- **Purpose:** Documents every output rendering mode this crate produces.
- **Responsibility:** One dedicated file per format — trigger, structure,
  rendering mechanism, and an example.
- **In Scope:** The 7 output shapes produced by `list`/`get`/`tags`/`tree` —
  6 selectable via [`format::`](../param/15_format.md) on the query
  commands, plus `tree`'s own fixed aligned layout.
- **Out of Scope:** Command-level syntax (→ [`../command/`](../command/readme.md)),
  the family's `plain_text` format — owned by `shader_chunks_compose`,
  reused by preview/render for their summary lines (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Format | Used By | Status |
|---|------|--------|---------|--------|
| 1 | [01_table_plain.md](01_table_plain.md) | table_plain | `.list` (default), `.get`, `.tags`; also `.tunables` in `shader_chunks_params` | ✅ |
| 2 | [02_tree_aligned.md](02_tree_aligned.md) | tree_aligned | `.tree` | ✅ |
| 4 | [04_markdown.md](04_markdown.md) | markdown | `.list`, `.get` | ✅ |
| 5 | [05_expanded.md](05_expanded.md) | expanded | `.get` (default), `.list` | ✅ |
| 6 | [06_json.md](06_json.md) | json | `.list`, `.get` | ✅ |
| 7 | [07_yaml.md](07_yaml.md) | yaml | `.list`, `.get` | ✅ |
| 8 | [08_names.md](08_names.md) | names | `.list`, `.get` | ✅ |

**Total:** 7 formats (of 8 across the `shader_chunks` family)

**Numbering note:** file #3 (`plain_text`) is not listed here — it moved
to [`shader_chunks_compose/docs/cli/format/01_plain_text.md`](../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md)
when the CLI split into per-command-group crates; the gap is preserved
rather than renumbering the remaining files, to avoid re-churning every
inbound cross-reference to #4–#8.

**Stability note:** `expanded` (fields in projection order) and `names`
(exactly the names) have deterministic layouts; `json`/`yaml` guarantee
*record* order but not *key* order within a record — parse, don't
string-match.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Commands producing these formats |

### Tests

| File | Relationship |
|------|--------------|
| [../../../../shader_chunks/tests/cli_subprocess_test.rs](../../../../shader_chunks/tests/cli_subprocess_test.rs) | Subprocess assertions on actual rendered output content |
