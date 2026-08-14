# Parameter Group Tests

### Scope

- **Purpose:** Corner-case test specifications for every parameter group `shader_chunks` declares — parameter *interactions*, not single-parameter edges.
- **Responsibility:** One file per parameter group, cross-referencing the real test functions covering member co-occurrence and interaction rules.
- **In Scope:** The 3 groups in [`../../../../docs/cli/param_group/`](../../../../docs/cli/param_group/readme.md).
- **Out of Scope:** Single-parameter edge cases (→ [`../param/`](../param/readme.md)); cross-command group invariants (→ [`../command_group/`](../command_group/readme.md)).

---

### Overview Table

| # | File | Parameter Group | Status |
|---|------|-----------------|--------|
| 1 | [01_filtering.md](01_filtering.md) | filtering | ✅ |
| 2 | [02_projection.md](02_projection.md) | projection | ✅ |
| 3 | [03_formatting.md](03_formatting.md) | formatting | ✅ |

**Total:** 3 parameter group test specs
