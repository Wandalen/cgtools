# Parameter :: 10. exports

- **Fundamental Type:** `String` (unilang `Kind::String`)
- **Constraints:** None on the value itself — any string is a valid
  substring needle; an empty value matches every chunk with at least the
  empty substring in a signature (i.e. all)
- **Default:** off (no export filtering)
- **Purpose:** Keeps only chunks where *any* export signature contains
  the given substring — find who provides a function without knowing
  which chunk. Case-insensitive by default; [`case::1`](04_case.md)
  makes it exact-case.

### Examples
```bash
# Valid values
list exports::fn format::names          # every chunk — each exports at least one fn
list exports::hash21 format::names      # hash21 — `fn hash21(p: vec2f) -> f32`
list exports::HASH21 case::1            # (empty — no exact-case match)

# No invalid values — a non-matching needle yields empty output, exit 0.
```

### Notes
- Matches against each full signature string (e.g.
  `fn hash21(p: vec2f) -> f32`), so return types and parameter types are
  searchable too (`exports::vec2f`).
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
