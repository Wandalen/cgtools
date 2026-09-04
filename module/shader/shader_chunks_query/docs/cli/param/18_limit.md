# Parameter :: 18. limit

- **Fundamental Type:**
  [`NonNegativeInteger`](../type/08_non_negative_integer.md) (unilang
  `Kind::Integer`, then `usize::try_from` in `shader_chunks_cli_core/src/lib.rs`)
- **Constraints:** ≥ 0; a negative value is `CliError::InvalidParam`
  ("allowed: a non-negative integer"), non-zero exit; a non-numeric value
  is rejected by unilang's integer coercion first
- **Default:** `0` — the reserved "unlimited" value, not "keep nothing"
- **Purpose:** Keeps at most N chunks of the sorted, offset result —
  paging's page size.

### Examples
```bash
# Valid values
list limit::2 format::names             # hash21, value_noise
list limit::2 offset::1 format::names   # value_noise, fbm3
list limit::0 count::0                  # unlimited — every chunk renders

# Invalid values (rejected with error)
list limit::-1    # "invalid `limit` value: `-1` (allowed: a non-negative integer)"
list limit::two   # unilang integer coercion failure, non-zero exit
```

### Notes
- Applies after [`sort::`](16_sort.md)/[`order::`](17_order.md) and after
  [`offset::`](19_offset.md) — the slice is
  `sorted[offset .. offset+limit]`.
- Ignored under [`count::1`](14_count.md) — the count is the pre-paging
  total.
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `0` (unlimited) | Page size |
| 2 | [.get](../command/02_get.md) | `0` (unlimited) | Page size |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [NonNegativeInteger](../type/08_non_negative_integer.md) | Integer | `usize` | ≥ 0, loud rejection of negatives |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
