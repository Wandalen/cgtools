# Parameter Tests

### Scope

- **Purpose:** Edge-case test specifications for every parameter owned
  by this crate.
- **Responsibility:** One file per parameter, cross-referencing the real
  test functions covering its boundary conditions.
- **In Scope:** The 1 parameter declared in [`../../../docs/cli/param/`](../../../docs/cli/param/readme.md).
- **Out of Scope:** Command-level integration scenarios (→ [`../command/`](../command/readme.md));
  the shared `names`/`transitive` parameters, owned by
  [`shader_chunks_query`](../../../../../shader_chunks_query/tests/docs/cli/param/readme.md).

---

### Overview Table

| # | File | Parameter | Status |
|---|------|-----------|--------|
| 1 | [01_out.md](01_out.md) | `out` | ✅ |

**Total:** 1 parameter test spec owned by this crate (of 32 across the
`shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/param/readme.md`](../../../docs/cli/param/readme.md) | Parameter documentation source |
