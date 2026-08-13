# Parameter Groups

### Scope

- **Purpose:** Documents every set of co-occurring parameters shared across commands.
- **Responsibility:** One dedicated file per group — membership, co-occurrence rationale, interaction rules.
- **In Scope:** The 3 groups partitioning the 19 named query parameters shared verbatim by `.list` and `.get`.
- **Out of Scope:** Per-parameter type/constraint/default detail (→ [`../param/`](../param/readme.md)).

---

### Overview Table

| # | File | Group | Parameters | Status |
|---|------|-------|------------|--------|
| 1 | [01_filtering.md](01_filtering.md) | filtering | 10 (`pattern` … `leaves`) | ✅ |
| 2 | [02_projection.md](02_projection.md) | projection | 2 (`fields`, `count`) | ✅ |
| 3 | [03_formatting.md](03_formatting.md) | formatting | 7 (`format` … `width`) | ✅ |

**Total:** 3 parameter groups

**Partition note:** the 3 groups cover the 19 *named* query parameters
exactly once each. The positional [`names`](../param/02_names.md) selects
the candidate set before any group applies, and [`name`](../param/01_name.md)
(`.tree` only) is outside the query engine entirely — neither belongs to a
group.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../param/readme.md](../param/readme.md) | Individual parameter definitions |
| [../command_group/01_query.md](../command_group/01_query.md) | The command group whose engine these groups shape |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/param_group/readme.md](../../../tests/docs/cli/param_group/readme.md) | Group-level test specifications |
