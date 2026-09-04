# Parameter Group :: 2. projection

### Pattern

Column selection: after [filtering](01_filtering.md) fixes *which chunks*
remain, this group fixes *what is shown about each* — a chosen subset of
the 7 queryable fields, or just the bare match count. Both members
co-occur on `.list` and `.get` with identical semantics; only the
`fields::` default differs per command.

### Members

| # | Parameter | Type | Default (`list`) | Default (`get`) |
|---|-----------|------|------------------|------------------|
| 1 | [`fields`](../param/13_fields.md) | List of [FieldName](../type/02_field_name.md) | `name,description,tags,depends_on` | `name,description,stage,tags,depends_on,exports` |
| 2 | [`count`](../param/14_count.md) | [Switch](../type/07_switch.md) | `false` | `false` |

### Interaction Rules

- `count::1` short-circuits the pipeline after filtering: the output is
  the matched-chunk total, taken *before* `offset::`/`limit::` paging and
  unaffected by `fields::`, `format::`, `sort::`, or `order::`.
- `fields::` order is projection order — columns render in the order
  given, duplicates allowed.
- An unknown field name fails loudly (`CliError::UnknownField`, listing
  the valid set) rather than rendering an empty column.
- `format::names` ignores `fields::` entirely — it always prints exactly
  the chunk names, one per line.

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`.list`](../command/01_list.md) | Overview default: 4 columns |
| 2 | [`.get`](../command/02_get.md) | Detail default: 6 columns |

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/param_group/02_projection.md`](../../../tests/docs/cli/param_group/02_projection.md) | Group-level test specification |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
