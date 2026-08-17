# Parameter Tests

### Scope

- **Purpose:** Edge-case test specifications for every parameter this
  crate introduces.
- **Responsibility:** One file per parameter, cross-referencing the real
  test functions covering its boundary conditions.
- **In Scope:** The 2 parameters in [`../../../docs/cli/param/`](../../../docs/cli/param/readme.md).
- **Out of Scope:** Command-level integration scenarios (→ [`../command/`](../command/readme.md));
  the `name` parameter's own edge cases — owned by
  [`shader_chunks_query`](../../../../../shader_chunks_query/tests/docs/cli/param/01_name.md).

---

### Overview Table

| # | File | Parameter | Status |
|---|------|-----------|--------|
| 1 | [01_file.md](01_file.md) | `file` | ✅ |
| 2 | [02_serve.md](02_serve.md) | `serve` | ✅ |

**Total:** 2 parameter test specs (of 32 across the `shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/param/readme.md`](../../../docs/cli/param/readme.md) | Parameter documentation source |
