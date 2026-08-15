# Parameter :: 9. transitive

- **Fundamental Type:** [`Switch`](../type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion); anything else is rejected by unilang before the
  command routine runs
- **Default:** `false` (direct dependents only on `.list`/`.get`; strict
  named set on `.compose`)
- **Purpose:** Modifier with a per-command meaning. On `.list`/`.get` it
  widens [`depends_on::`](08_depends_on.md) from direct dependents to the
  transitive closure (every chunk whose dependency chain reaches the given
  chunk) — a no-op without `depends_on::`. On `.compose` it widens the
  named set itself to its full dependency closure, so a single root name
  composes without spelling out its chain.

### Examples
```bash
# Valid values
list depends_on::hash21 format::names                # value_noise (direct)
list depends_on::hash21 transitive::1 format::names  # value_noise, fbm3, domain_warp
compose fbm3 transitive::1     # pulls value_noise and hash21 unasked
compose fbm3                   # strict default: missing-dependency error, exit 1

# Invalid values (rejected with error)
list transitive::maybe   # unilang boolean coercion failure, non-zero exit
```

### Notes
- Both closure walks (`reaches` in `shader_chunks_query_core/src/lib.rs`
  and `chunks_compose` in `shader_chunks_compose/src/lib.rs`) are
  cycle-safe — a `seen` set guards each worklist, so even a hypothetical
  cyclic registry cannot hang them.
- On `.compose` the widened set feeds the same
  `shader_chunks_core::try_compose` topological sort as an explicitly
  spelled-out set — `compose fbm3 transitive::1` and
  `compose hash21 value_noise fbm3` produce byte-identical output.
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `false` | Modifier for `depends_on::` |
| 2 | [.get](../command/02_get.md) | `false` | Modifier for `depends_on::` |
| 3 | [.compose](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | `false` | Widens the named set to its dependency closure |

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
