# shader_chunks_render CLI Documentation

Command/parameter/type reference for the `render` command of the
`shader_chunks` terminal tool — rendering one headless-GPU frame of a
bundled WGSL shader chunk's preview bundle to a static PNG. The engine
lives in
[`shader_chunks_render_core`](../../../shader_chunks_render_core/readme.md);
this crate wires it to the CLI via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
`render` accepts exactly one target — a bundled chunk `name` (owned by
[`shader_chunks_query`](../../../shader_chunks_query/docs/cli/readme.md))
or an arbitrary `file::` path (owned by
[`shader_chunks_preview`](../../../shader_chunks_preview/docs/cli/readme.md))
— builds and naga-validates the same preview bundle `preview` builds,
then captures one frame at the requested `size`/`time` to `out`. This
crate is one of 5 leaf CLIs assembled by the
[`shader_chunks`](../../../shader_chunks/docs/cli/readme.md) aggregator
— see that readme for the family-wide 8-command list and Scope
Decisions.

## Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | — | — | Complete |
| command/ | ✅ | ✅ | ✅ | — | — | Complete |
| command_group/ | ✅ | ✅ | ✅ | — | — | Complete |
| param/ | ✅ | ✅ | ✅ | — | — | Complete |
| type/ | ✅ | ✅ | ✅ | — | — | Complete |

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for this
crate's 1-command slice (`render`) of the `shader_chunks` family; no
incomplete-content placeholders; every command/command_group/param/type
instance has full required content and cross-references. No
`param_group/` or `format/` directory — `render`'s params belong to no
co-occurrence group, and it reuses `plain_text`
(owned by [`shader_chunks_compose`](../../../shader_chunks_compose/docs/cli/readme.md))
for its summary line rather than introducing a new format.
**Implementation Status:** Matches shipped code — `src/lib.rs` (CLI
wiring, contributes `help_groups()`/`help_examples()`/`commands()` to
the aggregator) and `src/bin/shader_chunks_render.rs`; engine in
[`shader_chunks_render_core`](../../../shader_chunks_render_core/readme.md)
(`tests/render_core_test.rs`); CLI-level tests in
`tests/render_cli_test.rs`. A right-sized `tests/docs/cli/` mirror
exists ([`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md))
covering param/command/command_group/type tiers.

## Navigation

- [`command/`](command/readme.md) — the 1 command (`render`)
- [`command_group/`](command_group/readme.md) — the 1 command group (`Render`)
- [`param/`](param/readme.md) — the 4 parameters owned by this crate (`out`, `size`, `time`, `set`)
- [`type/`](type/readme.md) — the 2 semantic types owned by this crate (`Float`, `ParameterOverride`)
- [`../../../shader_chunks/docs/cli/dictionary.md`](../../../shader_chunks/docs/cli/dictionary.md) — family-wide domain term glossary
- [`../../../shader_chunks/docs/cli/procedure.md`](../../../shader_chunks/docs/cli/procedure.md) — how to extend a `docs/cli/` tree when an entity is added or removed
- [`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md) — test specification mirror
- [`../../readme.md`](../../readme.md) — crate readme (purpose, examples, links back here)
- [`../../../shader_chunks/docs/cli/readme.md`](../../../shader_chunks/docs/cli/readme.md) — family index (all 5 leaf CLIs, all 8 commands)

## Scope Decisions

This crate inherits the family-wide Scope Decisions stated in
[the family index](../../../shader_chunks/docs/cli/readme.md#scope-decisions)
(`user_story/`, `command_noun`/`command_verb`, `env_param.md`/
`config_param.md`, and `index.md` omissions) — none of those rationales
are specific to this crate, so they are not restated here. Two
crate-local addenda:

- **`dictionary.md`/`procedure.md` omitted here.** Both stay centralized
  at the family index
  ([`dictionary.md`](../../../shader_chunks/docs/cli/dictionary.md),
  [`procedure.md`](../../../shader_chunks/docs/cli/procedure.md)) rather
  than being duplicated per leaf — one glossary and one extension
  procedure serve all 5 CLIs since they share the same entity taxonomy
  (`cli_doc_des.rulebook.md`).
- **`param_group/` and `format/` omitted here.** `render`'s 6 parameters
  (the shared `name`/`file` target pair plus this crate's own `out`,
  `size`, `time`, `set`) are target selection and artifact shaping, not
  filter/projection/formatting — no co-occurrence group applies. Its
  output is a binary PNG file plus a `plain_text` summary line reused
  verbatim from `shader_chunks_compose` — no new format is introduced.
