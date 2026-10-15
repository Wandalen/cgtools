# Parameter :: 22. reverse

- **Fundamental Type:** [`Switch`](../type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion); anything else is rejected by unilang before the
  command routine runs
- **Default:** `false` (forward dependency tree)
- **Purpose:** Modifier for `.tree` only — no other command accepts it.
  Flips the walk direction: instead of showing what a chunk depends on,
  shows what (transitively) depends on it — the dependents chain instead
  of the dependency chain.

### Examples
```bash
# Valid values
tree hash21                  # forward (default): hash21's own dependencies (none -- it's a leaf)
tree hash21 reverse::1       # reverse: value_noise, then fbm3 -- everything that (transitively) depends on hash21
tree reverse::1              # reverse forest: one tree per leaf chunk, each showing its dependents

# Invalid values (rejected with error)
tree hash21 reverse::maybe   # unilang boolean coercion failure, non-zero exit
```

### Notes
- Forward mode's forest roots are "nothing depends on me"
  ([`dependents_free_roots`](../../../../shader_chunks_query_core/src/lib.rs) —
  chunks like `domain_warp`/`fullscreen_triangle`; `fbm3` is not one of
  them — `domain_warp` itself depends on it, per the `reverse::1` example
  above); reverse mode's forest roots are "I depend on nothing"
  (`leaf_roots` — chunks like `hash21`/`fullscreen_triangle`), since a
  reverse walk has no forward "root" of its own to start from.
- Backed by `shader_chunks_query_core::reverse_adjacency`, a
  `HashMap<&str, Vec<&str>>` built by inverting every chunk's
  `depends_on` edges once per call — not memoized, since the bundled
  chunk set is small and `tree` is not a hot path.
- One recursive walk function (`dep_tree_node`) backs both directions via
  an injected `children_of` closure — no separate reverse-tree code path
  to keep in sync with the forward one.
- Member of no parameter group — [`transitive::`](09_transitive.md) is
  the closest sibling in spirit (both are graph-walk Switch modifiers),
  but it belongs to the shared `.list`/`.get` filtering group, while
  `reverse` is `.tree`-only.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.tree](../command/04_tree.md) | `false` | Flips dependency walk to dependents walk |

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
