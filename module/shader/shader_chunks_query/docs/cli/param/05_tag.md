# Parameter :: 5. tag

- **Fundamental Type:** list of [`TagSelector`](../type/09_tag_selector.md)
  (unilang `Kind::List(String, ',')` — one `tag::` value, comma-separated)
- **Constraints:** Each selector is either a `group:tag` pair (matched
  exactly against a declared tag) or a bare `tag` (matched against the
  tag part under *any* group); selectors that match nothing yield empty
  output, not an error
- **Default:** off (no `tag::` means no tag filtering)
- **Purpose:** Keeps only chunks carrying matching tags. Multiple
  selectors combine per [`tags_mode::`](06_tags_mode.md) — union (`any`,
  default) or intersection (`all`).

### Examples
```bash
# Valid values
list tag::noise format::names                     # bare: value_noise, fbm3, value_noise3, gradient_noise, voronoi, domain_warp
list tag::category:noise format::names            # pair: same two
list tag::noise,hash format::names                # any-of: + hash21
list tag::noise,fractal tags_mode::all            # all-of: fbm3 only

# No invalid values — a selector matching no declared tag yields empty
# output with exit 0 (discover declared tags via `tags`).
```

### Notes
- `tags` (the command) is the discovery counterpart: it lists every
  declared `group:tag` pair this parameter can select on.
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
| [TagSelector](../type/09_tag_selector.md) | String (list) | `Vec<String>` | `group:tag` exact pair or bare `tag` any-group |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
