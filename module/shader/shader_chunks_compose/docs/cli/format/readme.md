# Formats

### Scope

- **Purpose:** Documents every output rendering mode this crate produces.
- **Responsibility:** One dedicated file per format — trigger, structure,
  rendering mechanism, and an example.
- **In Scope:** The 1 output shape this crate introduces — `plain_text`,
  reused by `.preview` and `.render` (in their own crates) for their
  summary lines.
- **Out of Scope:** Command-level syntax (→ [`../command/`](../command/readme.md)),
  the family's other 9 formats — owned by `shader_chunks_query` (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Format | Used By | Status |
|---|------|--------|---------|--------|
| 1 | [01_plain_text.md](01_plain_text.md) | plain_text | `.compose` (composed WGSL); `.preview`, `.render` (summary lines, in their own crates) | ✅ |

**Total:** 1 format (of 10 across the `shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Commands producing this format |

### Tests

| File | Relationship |
|------|--------------|
| [../../../../shader_chunks/tests/cli_subprocess_test.rs](../../../../shader_chunks/tests/cli_subprocess_test.rs) | Subprocess assertions on actual rendered output content |
