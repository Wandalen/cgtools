# Type Tests

### Scope

- **Purpose:** Construction, parsing, and rejection test specifications
  for every type owned by this crate.
- **Responsibility:** One file per type, documenting the abstract
  contract independent of any single parameter's usage context.
- **In Scope:** The 1 type in [`../../../docs/cli/type/`](../../../docs/cli/type/readme.md).
- **Out of Scope:** Parameter-specific usage constraints (→ [`../param/`](../param/readme.md));
  the family's other 10 types, owned by
  [`shader_chunks_query`](../../../../../shader_chunks_query/tests/docs/cli/type/readme.md).

---

### Overview Table

| # | File | Type | Status |
|---|------|------|--------|
| 1 | [01_float.md](01_float.md) | Float | ✅ |

**Total:** 1 type test spec owned by this crate (of 11 across the
`shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/type/readme.md`](../../../docs/cli/type/readme.md) | Type documentation source |
