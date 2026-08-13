# Parameter :: 7. stage

- **Fundamental Type:** [`StageSelector`](../type/10_stage_selector.md)
  (unilang `Kind::String`)
- **Constraints:** `any` (no stage filtering), `none` (only chunks with
  no declared stage), or any literal stage name matched exactly against
  the chunk's `//@ stage:` metadata; a literal matching no chunk yields
  empty output, not an error
- **Default:** `any`
- **Purpose:** Keeps only chunks declared for the given pipeline stage —
  or explicitly stage-agnostic ones via `none`.

### Examples
```bash
# Valid values
list stage::none format::names       # hash21, value_noise, fbm3
list stage::vertex format::names     # fullscreen_triangle
list stage::fragment format::names   # (empty — nothing declares fragment)

# No invalid values — `stage::` is a selector, not a closed enum; an
# unmatched literal yields empty output with exit 0.
```

### Notes
- `none` is a reserved selector word, not a stage name — a chunk cannot
  declare `//@ stage: none` and still be selectable as literal (the
  selector word wins). No bundled chunk does.
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `any` | Narrows the full registry |
| 2 | [.get](../command/02_get.md) | `any` | Narrows the named selection |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [StageSelector](../type/10_stage_selector.md) | String | `String` | `any` \| `none` \| literal stage name |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
