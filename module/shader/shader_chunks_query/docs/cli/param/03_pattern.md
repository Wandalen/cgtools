# Parameter :: 3. pattern

- **Fundamental Type:** `String` (unilang `Kind::String`)
- **Constraints:** None on the value itself — any string is a valid
  substring needle; an empty value matches every chunk
- **Default:** off (no `pattern::` means no name filtering)
- **Purpose:** Keeps only chunks whose *name* contains the given
  substring. Case-insensitive by default; [`case::1`](04_case.md) makes
  the match exact-case.

### Examples
```bash
# Valid values
list pattern::noise format::names       # value_noise, value_noise3, gradient_noise
list pattern::NOISE format::names       # same — insensitive by default
list pattern::NOISE case::1             # (no output — no exact-case match)

# No invalid values — any string is a legal needle; a non-matching one
# yields empty output with exit 0, not an error.
```

### Notes
- Matches the `name` field only — use [`exports::`](10_exports.md) to
  search export signatures and [`tag::`](05_tag.md) for categories.
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group; combines conjunctively with every other filter.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | off | Narrows the full registry |
| 2 | [.get](../command/02_get.md) | off | Narrows the named selection |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| — (plain string) | String | `String` | None — any substring needle |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
