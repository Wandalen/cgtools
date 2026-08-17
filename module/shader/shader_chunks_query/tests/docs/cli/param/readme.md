# Parameter Tests

### Scope

- **Purpose:** Edge-case test specifications for every parameter this
  crate's commands accept.
- **Responsibility:** One file per parameter, cross-referencing the real
  test functions covering its boundary conditions.
- **In Scope:** The 21 parameters declared in [`../../../docs/cli/param/`](../../../docs/cli/param/readme.md).
- **Out of Scope:** Group-level interaction scenarios (→ [`../param_group/`](../param_group/readme.md));
  command-level integration scenarios (→ [`../command/`](../command/readme.md));
  type-level construction/parsing (→ [`../type/`](../type/readme.md)).

---

### Overview Table

| # | File | Parameter | Status |
|---|------|-----------|--------|
| 1 | [01_name.md](01_name.md) | `name` | ✅ |
| 2 | [02_names.md](02_names.md) | `names` | ✅ |
| 3 | [03_pattern.md](03_pattern.md) | `pattern` | ✅ |
| 4 | [04_case.md](04_case.md) | `case` | ✅ |
| 5 | [05_tag.md](05_tag.md) | `tag` | ✅ |
| 6 | [06_tags_mode.md](06_tags_mode.md) | `tags_mode` | ✅ |
| 7 | [07_stage.md](07_stage.md) | `stage` | ✅ |
| 8 | [08_depends_on.md](08_depends_on.md) | `depends_on` | ✅ |
| 9 | [09_transitive.md](09_transitive.md) | `transitive` | ✅ |
| 10 | [10_exports.md](10_exports.md) | `exports` | ✅ |
| 11 | [11_roots.md](11_roots.md) | `roots` | ✅ |
| 12 | [12_leaves.md](12_leaves.md) | `leaves` | ✅ |
| 13 | [13_fields.md](13_fields.md) | `fields` | ✅ |
| 14 | [14_count.md](14_count.md) | `count` | ✅ |
| 15 | [15_format.md](15_format.md) | `format` | ✅ |
| 16 | [16_sort.md](16_sort.md) | `sort` | ✅ |
| 17 | [17_order.md](17_order.md) | `order` | ✅ |
| 18 | [18_limit.md](18_limit.md) | `limit` | ✅ |
| 19 | [19_offset.md](19_offset.md) | `offset` | ✅ |
| 20 | [20_heading.md](20_heading.md) | `heading` | ✅ |
| 21 | [21_width.md](21_width.md) | `width` | ✅ |

**Total:** 21 parameter test specs (of 26 across the `shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/param/readme.md`](../../../docs/cli/param/readme.md) | Parameter documentation source |
