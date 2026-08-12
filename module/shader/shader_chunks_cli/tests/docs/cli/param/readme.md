# Parameter Tests

### Scope

- **Purpose:** Edge-case test specifications for every parameter `shader_chunks_cli` accepts.
- **Responsibility:** One file per parameter, cross-referencing the real test functions covering its boundary conditions.
- **In Scope:** The 2 parameters declared in [`../../../../docs/cli/param/`](../../../../docs/cli/param/readme.md).
- **Out of Scope:** Command-level integration scenarios (→ [`../command/`](../command/readme.md)); type-level construction/parsing (→ [`../type/`](../type/readme.md)).

---

### Overview Table

| # | File | Parameter | Status |
|---|------|-----------|--------|
| 1 | [01_name.md](01_name.md) | `name` | ✅ |
| 2 | [02_names.md](02_names.md) | `names` | ✅ |

**Total:** 2 parameter test specs
