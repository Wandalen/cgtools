# Parameter :: 4. case

- **Fundamental Type:** [`Switch`](../type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion, case-insensitive); anything else is rejected by
  unilang before the command routine runs
- **Default:** `false` (insensitive matching)
- **Purpose:** Modifier — makes [`pattern::`](03_pattern.md) and
  [`exports::`](10_exports.md) matching case-sensitive. Carries no filter
  of its own; with neither of those set it is a no-op.

### Examples
```bash
# Valid values
list pattern::NOISE case::1 format::names   # (empty — no exact-case match)
list exports::FN case::0 format::names      # all 4 — insensitive matches `fn`

# Invalid values (rejected with error)
list case::maybe    # unilang boolean coercion failure, non-zero exit
```

### Notes
- Deliberately one shared switch for both substring filters rather than
  `pattern_case::`/`exports_case::` — the query surface stays flat and
  the two filters are never usefully mixed-sensitivity.
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `false` | Modifier for `pattern::`/`exports::` |
| 2 | [.get](../command/02_get.md) | `false` | Modifier for `pattern::`/`exports::` |

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
