# Parameter Tests

### Scope

- **Purpose:** Edge-case test specifications for every parameter owned
  by this crate.
- **Responsibility:** One file per parameter, cross-referencing the real
  test functions covering its boundary conditions.
- **In Scope:** The 3 parameters declared in [`../../../docs/cli/param/`](../../../docs/cli/param/readme.md).
- **Out of Scope:** Command-level integration scenarios (→ [`../command/`](../command/readme.md));
  type-level construction/parsing (→ [`../type/`](../type/readme.md));
  the shared `name`/`file` target parameters, owned by
  [`shader_chunks_query`](../../../../../shader_chunks_query/tests/docs/cli/param/01_name.md)
  and
  [`shader_chunks_preview`](../../../../../shader_chunks_preview/tests/docs/cli/param/01_file.md)
  respectively.

---

### Overview Table

| # | File | Parameter | Status |
|---|------|-----------|--------|
| 1 | [01_out.md](01_out.md) | `out` | ✅ |
| 2 | [02_size.md](02_size.md) | `size` | ✅ |
| 3 | [03_time.md](03_time.md) | `time` | ✅ |

**Total:** 3 parameter test specs owned by this crate (of 26 across the
`shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/param/readme.md`](../../../docs/cli/param/readme.md) | Parameter documentation source |
