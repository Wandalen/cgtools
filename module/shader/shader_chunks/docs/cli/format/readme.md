# Formats

### Scope

- **Purpose:** Documents every output rendering mode `shader_chunks` produces.
- **Responsibility:** One dedicated file per format — trigger, structure, rendering mechanism, and an example.
- **In Scope:** The 8 output shapes produced across the 6 commands — 6 selectable via [`format::`](../param/15_format.md) on the query commands, plus the fixed shapes of `.tree` and `.compose` (`.tunables` reuses `table_plain`, see row 1).
- **Out of Scope:** Command-level syntax (→ [`../command/`](../command/readme.md)).

---

### Overview Table

| # | File | Format | Used By | Status |
|---|------|--------|---------|--------|
| 1 | [01_table_plain.md](01_table_plain.md) | table_plain | `.list` (default), `.get`, `.tags`, `.tunables` | ✅ |
| 2 | [02_tree_aligned.md](02_tree_aligned.md) | tree_aligned | `.tree` | ✅ |
| 3 | [03_plain_text.md](03_plain_text.md) | plain_text | `.compose` | ✅ |
| 4 | [04_markdown.md](04_markdown.md) | markdown | `.list`, `.get` | ✅ |
| 5 | [05_expanded.md](05_expanded.md) | expanded | `.get` (default), `.list` | ✅ |
| 6 | [06_json.md](06_json.md) | json | `.list`, `.get` | ✅ |
| 7 | [07_yaml.md](07_yaml.md) | yaml | `.list`, `.get` | ✅ |
| 8 | [08_names.md](08_names.md) | names | `.list`, `.get` | ✅ |

**Total:** 8 formats

**Stability note:** `expanded` (fields in projection order) and `names`
(exactly the names) have deterministic layouts; `json`/`yaml` guarantee
*record* order but not *key* order within a record — parse, don't
string-match.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../command/readme.md](../command/readme.md) | Commands producing these formats |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/cli_subprocess_test.rs](../../../tests/cli_subprocess_test.rs) | Subprocess assertions on actual rendered output content |
