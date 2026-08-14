# Parameter :: 19. offset

- **Fundamental Type:**
  [`NonNegativeInteger`](../type/08_non_negative_integer.md) (unilang
  `Kind::Integer`, then `usize::try_from` in `src/cli.rs`)
- **Constraints:** ≥ 0; a negative value is `CliError::InvalidParam`
  ("allowed: a non-negative integer"), non-zero exit; a non-numeric value
  is rejected by unilang's integer coercion first
- **Default:** `0` (start at the first chunk)
- **Purpose:** Skips the first N chunks of the sorted result — paging's
  starting position.

### Examples
```bash
# Valid values
list offset::1 format::names            # value_noise, fbm3, fullscreen_triangle
list offset::1 limit::2 format::names   # value_noise, fbm3
list offset::9 format::names            # (empty — past the end, exit 0)

# Invalid values (rejected with error)
list offset::-1   # "invalid `offset` value: `-1` (allowed: a non-negative integer)"
```

### Notes
- An offset past the end of the result yields empty output with exit 0 —
  paging is never an error, matching how an over-narrow filter behaves.
- Ignored under [`count::1`](14_count.md).
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `0` | Page start |
| 2 | [.get](../command/02_get.md) | `0` | Page start |

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
