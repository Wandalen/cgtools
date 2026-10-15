# Parameter :: 14. count

- **Fundamental Type:** [`Switch`](../type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion); anything else is rejected by unilang before the
  command routine runs
- **Default:** `false` (render matched chunks, not their count)
- **Purpose:** Prints only the number of chunks surviving the filters —
  the query's aggregate answer instead of its rows.

### Examples
```bash
# Valid values
list count::1                    # 4
list pattern::noise count::1     # 2
list count::1 limit::1           # 4 — count is taken BEFORE paging

# Invalid values (rejected with error)
list count::maybe   # unilang boolean coercion failure, non-zero exit
```

### Notes
- Short-circuits the pipeline after filtering: `fields::`, `format::`,
  `sort::`, `order::`, `limit::`, `offset::`, `heading::`, `width::` are
  all accepted and ignored when `count::1` — the total is a property of
  the filtered set, not of any rendering of it.
- Member of the [projection](../param_group/02_projection.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `false` | Aggregate over the full registry |
| 2 | [.get](../command/02_get.md) | `false` | Aggregate over the named selection |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [Switch](../type/07_switch.md) | Boolean | `bool` | `1/true/yes` vs `0/false/no` |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
