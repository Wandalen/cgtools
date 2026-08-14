# Parameter :: 17. order

- **Fundamental Type:** [`SortOrder`](../type/05_sort_order.md) (unilang
  `Kind::String`, parsed by `SortOrder::from_str` in `src/lib.rs`)
- **Constraints:** Exactly `asc` or `desc`; anything else is
  `CliError::InvalidParam` naming the allowed set, non-zero exit
- **Default:** `asc`
- **Purpose:** Direction modifier for [`sort::`](16_sort.md) — `desc`
  reverses the sorted sequence, *including* `sort::input` (reversed
  selection order).

### Examples
```bash
# Valid values
list sort::name order::desc format::names   # value_noise, hash21, ...
list order::desc format::names              # reversed registry order

# Invalid values (rejected with error)
list order::bogus   # "invalid `order` value: `bogus` (allowed: asc, desc)"
```

### Notes
- Applies after the full sort (including tie-breaking), so `desc` is an
  exact reversal — ties reverse too.
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `asc` | Direction for `sort::` |
| 2 | [.get](../command/02_get.md) | `asc` | Direction for `sort::` |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [SortOrder](../type/05_sort_order.md) | String (enum) | `SortOrder` | `asc` \| `desc`, loud rejection otherwise |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
