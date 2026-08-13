# Parameter Group :: 1. filtering

### Pattern

Row selection: every parameter in this group narrows *which chunks* the
query keeps, before projection and formatting apply. All members co-occur
on the same two commands (`.list`, `.get`) with identical names, types,
and semantics — only the candidate set they narrow differs (`get` starts
from the named chunks, `list` from every chunk).

### Members

| # | Parameter | Type | Default | Narrows by |
|---|-----------|------|---------|------------|
| 1 | [`pattern`](../param/03_pattern.md) | String | off | Substring of the chunk name |
| 2 | [`case`](../param/04_case.md) | [Switch](../type/07_switch.md) | `false` | Modifier — makes `pattern::`/`exports::` case-sensitive |
| 3 | [`tag`](../param/05_tag.md) | List of [TagSelector](../type/09_tag_selector.md) | off | Tag membership |
| 4 | [`tags_mode`](../param/06_tags_mode.md) | [TagsMode](../type/06_tags_mode.md) | `any` | Modifier — union vs. intersection of `tag::` selectors |
| 5 | [`stage`](../param/07_stage.md) | [StageSelector](../type/10_stage_selector.md) | `any` | Declared pipeline stage |
| 6 | [`depends_on`](../param/08_depends_on.md) | [ChunkName](../type/01_chunk_name.md) | off | Direct dependency on the given chunk |
| 7 | [`transitive`](../param/09_transitive.md) | [Switch](../type/07_switch.md) | `false` | Modifier — widens `depends_on::` to the transitive closure |
| 8 | [`exports`](../param/10_exports.md) | String | off | Substring of any export signature |
| 9 | [`roots`](../param/11_roots.md) | [Switch](../type/07_switch.md) | `false` | Chunks nothing else depends on |
| 10 | [`leaves`](../param/12_leaves.md) | [Switch](../type/07_switch.md) | `false` | Chunks with no dependencies |

### Interaction Rules

- All active filters are conjunctive (AND): a chunk survives only if it
  passes every one.
- `case::` has no filter of its own — it modifies `pattern::` and
  `exports::` matching; on its own it is a no-op.
- `tags_mode::` is only observable when `tag::` carries ≥2 selectors.
- `transitive::` is only observable when `depends_on::` is set.
- `roots::1` and `leaves::1` may combine — the intersection is chunks that
  are both (currently `fullscreen_triangle`).
- An unknown `depends_on::` chunk fails loudly
  (`CliError::UnknownChunk`) rather than matching nothing silently.

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`.list`](../command/01_list.md) | Filters the full registry |
| 2 | [`.get`](../command/02_get.md) | Filters the named-chunk selection |

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/param_group/01_filtering.md`](../../../tests/docs/cli/param_group/01_filtering.md) | Group-level test specification |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
