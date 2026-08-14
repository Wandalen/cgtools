# Parameter :: 16. sort

- **Fundamental Type:** [`SortKey`](../type/04_sort_key.md) (unilang
  `Kind::String`, parsed by `SortKey::from_str` in `src/lib.rs`)
- **Constraints:** Exactly one of `input`, `name`, `stage`,
  `description`; anything else is `CliError::InvalidParam` naming the
  allowed set, non-zero exit
- **Default:** `input` (selection order — registry order for `list`,
  the `names` argument order for `get`)
- **Purpose:** Orders the filtered result before paging and rendering.

### Examples
```bash
# Valid values
list sort::name format::names          # fbm3, fullscreen_triangle, hash21, value_noise
list sort::stage format::names         # stage-less first (ties by name), vertex last
list sort::description format::names   # lexicographic on the description text

# Invalid values (rejected with error)
list sort::bogus   # "invalid `sort` value: `bogus` (allowed: input, name,
                    #  stage, description)"
```

### Notes
- `stage` and `description` sorts are deterministic under ties — the
  chunk name is the secondary key; stage-less chunks sort before staged
  ones (empty string first).
- Sorting precedes paging: `offset::`/`limit::` always slice the sorted
  sequence.
- Member of the [formatting](../param_group/03_formatting.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `input` | Registry order |
| 2 | [.get](../command/02_get.md) | `input` | `names` argument order |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [SortKey](../type/04_sort_key.md) | String (enum) | `SortKey` | 4 closed variants, loud rejection otherwise |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
