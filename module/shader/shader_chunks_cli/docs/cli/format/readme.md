# Formats

### Scope

- **Purpose:** Documents every output rendering mode `shader_chunks_cli` produces.
- **Responsibility:** One dedicated file per format — trigger, structure, rendering mechanism, and an example.
- **In Scope:** The 3 output shapes produced across the 5 commands.
- **Out of Scope:** Command-level syntax (→ [`../command/`](../command/readme.md)).

---

### Overview Table

| # | File | Format | Used By | Status |
|---|------|--------|---------|--------|
| 1 | [01_table_plain.md](01_table_plain.md) | table_plain | `.list`, `.tags` | ✅ |
| 2 | [02_tree_aligned.md](02_tree_aligned.md) | tree_aligned | `.tree` | ✅ |
| 3 | [03_plain_text.md](03_plain_text.md) | plain_text | `.get`, `.compose` | ✅ |

**Total:** 3 formats

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../command/readme.md](../command/readme.md) | Commands producing these formats |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/cli_subprocess_test.rs](../../../tests/cli_subprocess_test.rs) | Subprocess assertions on actual rendered output content |
