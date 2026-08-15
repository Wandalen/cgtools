# Parameter Group :: 3. formatting

### Pattern

Output shaping: after [filtering](01_filtering.md) and
[projection](02_projection.md) fix the result set and its columns, this
group fixes how the result is ordered, paged, and rendered. All members
co-occur on `.list` and `.get` with identical semantics; only the
`format::` default differs per command.

### Members

| # | Parameter | Type | Default (`list`) | Default (`get`) |
|---|-----------|------|------------------|------------------|
| 1 | [`format`](../param/15_format.md) | [OutputFormat](../type/03_output_format.md) | `table` | `expanded` |
| 2 | [`sort`](../param/16_sort.md) | [SortKey](../type/04_sort_key.md) | `input` | `input` |
| 3 | [`order`](../param/17_order.md) | [SortOrder](../type/05_sort_order.md) | `asc` | `asc` |
| 4 | [`limit`](../param/18_limit.md) | [NonNegativeInteger](../type/08_non_negative_integer.md) | `0` (unlimited) | `0` (unlimited) |
| 5 | [`offset`](../param/19_offset.md) | [NonNegativeInteger](../type/08_non_negative_integer.md) | `0` | `0` |
| 6 | [`heading`](../param/20_heading.md) | String | off | off |
| 7 | [`width`](../param/21_width.md) | [NonNegativeInteger](../type/08_non_negative_integer.md) | `0` (auto) | `0` (auto) |

### Interaction Rules

- Pipeline order is fixed: sort (`sort::`+`order::`) → page
  (`offset::` then `limit::`) → render (`format::` with
  `heading::`/`width::`). Paging always applies to the *sorted* sequence.
- `heading::` and `width::` shape only the `table` and `markdown` formats;
  under any other `format::` they are accepted and ignored (documented
  no-ops, not errors).
- `sort::input` preserves selection order — registry order for `list`,
  the `names` argument order for `get`; `order::desc` reverses whichever
  key is active, including `input`.
- `offset::` past the end of the result yields empty output with exit 0 —
  paging is never an error.
- Every enum-valued member rejects unknown values loudly
  (`CliError::InvalidParam` naming the allowed set).

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`.list`](../command/01_list.md) | `format::table` default |
| 2 | [`.get`](../command/02_get.md) | `format::expanded` default |

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/param_group/03_formatting.md`](../../../tests/docs/cli/param_group/03_formatting.md) | Group-level test specification |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
