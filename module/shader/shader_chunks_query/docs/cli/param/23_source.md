# Parameter :: 23. source

- **Fundamental Type:** `String` (unilang `Kind::String`)
- **Constraints:** None on the value itself — any string is a valid
  substring needle; an empty value matches every chunk with at least the
  empty substring in its body (i.e. all)
- **Default:** off (no source filtering)
- **Purpose:** Keeps only chunks whose raw WGSL body contains the given
  substring — find chunks by an internal constant, helper call, or code
  fragment without knowing chunk names or export signatures. Case-insensitive
  by default; [`case::1`](04_case.md) makes it exact-case.

### Examples
```bash
# Valid values
list source::33.33 format::names           # hash21, hash22, hash33 -- shared magic constant in their WGSL bodies
list source::"fn hash21( p" format::names  # hash21 -- matches inside the function body, not just its name
list source::"FN HASH21( P" case::1        # (empty -- no exact-case match; the body itself is lowercase)

# No invalid values — a non-matching needle yields empty output, exit 0.
```

### Notes
- Matches against the chunk's entire raw WGSL file text — the same text the
  `source` output field renders — including its `//@` manifest header
  comments, not just the function body (contrast [`exports::`](10_exports.md),
  which matches only export signature strings).
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

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
