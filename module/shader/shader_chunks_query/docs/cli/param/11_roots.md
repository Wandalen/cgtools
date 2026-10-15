# Parameter :: 11. roots

- **Fundamental Type:** [`Switch`](../type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion); anything else is rejected by unilang before the
  command routine runs
- **Default:** `false` (no root filtering)
- **Purpose:** Keeps only *root* chunks — chunks no other chunk depends
  on; the natural entry points `tree` renders as its forest.

### Examples
```bash
# Valid values
list roots::1 format::names            # fullscreen_triangle, hash33, ... — every chunk nothing depends on
list roots::1 leaves::1 format::names  # every standalone chunk (no deps and no dependents at once)

# Invalid values (rejected with error)
list roots::maybe   # unilang boolean coercion failure, non-zero exit
```

### Notes
- Computed from the registry's `depends_on` edges
  (`depended_on_set` in `shader_chunks_query_core/src/lib.rs`) — a chunk is a root iff its name
  appears in no other chunk's dependency list.
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
