# Parameter :: 6. tags_mode

- **Fundamental Type:** [`TagsMode`](../type/06_tags_mode.md) (unilang
  `Kind::String`, parsed by `TagsMode::from_str` in `shader_chunks_query_core/src/lib.rs`)
- **Constraints:** Exactly `any` or `all`; anything else is
  `CliError::InvalidParam` naming the allowed set, non-zero exit
- **Default:** `any` (union)
- **Purpose:** Modifier — chooses how multiple [`tag::`](05_tag.md)
  selectors combine: `any` keeps a chunk matching *at least one*
  selector, `all` demands *every* selector match. Only observable with
  ≥2 selectors.

### Examples
```bash
# Valid values
list tag::noise,hash tags_mode::any format::names   # union: 3 chunks
list tag::noise,fractal tags_mode::all format::names # intersection: fbm3

# Invalid values (rejected with error)
list tags_mode::bogus   # "invalid `tags_mode` value: `bogus` (allowed: any, all)"
```

### Notes
- With zero or one `tag::` selector both modes behave identically.
- Member of the [filtering](../param_group/01_filtering.md) parameter
  group.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `any` | Combines `tag::` selectors |
| 2 | [.get](../command/02_get.md) | `any` | Combines `tag::` selectors |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [TagsMode](../type/06_tags_mode.md) | String (enum) | `TagsMode` | `any` \| `all`, loud rejection otherwise |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
