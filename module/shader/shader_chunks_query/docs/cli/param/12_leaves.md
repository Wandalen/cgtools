# Parameter :: 12. leaves

- **Fundamental Type:** [`Switch`](../type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion); anything else is rejected by unilang before the
  command routine runs
- **Default:** `false` (no leaf filtering)
- **Purpose:** Keeps only *leaf* chunks — chunks with no dependencies of
  their own; the self-contained building blocks safe to compose alone.

### Examples
```bash
# Valid values
list leaves::1 format::names           # hash21, fullscreen_triangle
list roots::1 leaves::1 format::names  # fullscreen_triangle (both at once)

# Invalid values (rejected with error)
list leaves::maybe   # unilang boolean coercion failure, non-zero exit
```

### Notes
- Dual of [`roots::`](11_roots.md): `roots::` looks at inbound edges
  (who depends on me), `leaves::` at outbound edges (whom do I depend
  on). A chunk in neither set sits mid-chain (`value_noise`); a chunk in
  both is fully isolated (`fullscreen_triangle`).
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `false` | Narrows the full registry |
| 2 | [.get](../command/02_get.md) | `false` | Narrows the named selection |

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
