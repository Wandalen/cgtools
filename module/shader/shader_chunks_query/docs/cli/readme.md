# shader_chunks_query CLI Documentation

Command/parameter/type reference for the `list`, `get`, `tags`, and
`tree` commands of the `shader_chunks` terminal tool — querying,
inspecting, and graphing the WGSL shader chunks bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md). The
engine lives in
[`shader_chunks_query_core`](../../../shader_chunks_query_core/readme.md);
this crate wires it to the CLI via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
`list` and `get` share one engine with a 20-parameter filter/projection/
formatting surface (2 positional selectors + 19 shared named
parameters); `tags` lists every distinct tag; `tree` renders the
dependency graph. This crate is one of 5 leaf CLIs assembled by the
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
| param_group/ | ✅ | ✅ | ✅ | — | — | Complete |
| type/ | ✅ | ✅ | ✅ | — | — | Complete |
| format/ | ✅ | ✅ | ✅ | — | — | Complete |

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for this
crate's 4-command slice (`list`, `get`, `tags`, `tree`) of the
`shader_chunks` family; no incomplete-content placeholders; every
command/command_group/param/param_group/type/format instance has full
required content and cross-references.
**Implementation Status:** Matches shipped code — `src/lib.rs` (CLI
wiring, contributes `help_groups()`/`help_examples()`/`commands()` to
the aggregator) and `src/bin/shader_chunks_query.rs`; engine in
[`shader_chunks_query_core`](../../../shader_chunks_query_core/readme.md)
(`tests/shader_chunks_query_core_test.rs`). A right-sized
`tests/docs/cli/` mirror exists
([`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md))
covering param/param_group/command/command_group/type tiers.

## Navigation

- [`command/`](command/readme.md) — the 4 commands (`list`, `get`, `tags`, `tree`)
- [`command_group/`](command_group/readme.md) — the 2 command groups (`Query`, `Graph`)
- [`param/`](param/readme.md) — the 21 parameters (2 positional selectors + 19 shared named query parameters)
- [`param_group/`](param_group/readme.md) — the 3 parameter groups (`filtering`, `projection`, `formatting`)
- [`type/`](type/readme.md) — the 10 semantic types (`ChunkName`, the query enums, selectors, `Switch`, `NonNegativeInteger`)
- [`format/`](format/readme.md) — the 7 output formats selectable via `format::`, plus `tree`'s own aligned layout
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
are specific to this crate, so they are not restated here. One
crate-local addendum:

- **`dictionary.md`/`procedure.md` omitted here.** Both stay centralized
  at the family index
  ([`dictionary.md`](../../../shader_chunks/docs/cli/dictionary.md),
  [`procedure.md`](../../../shader_chunks/docs/cli/procedure.md)) rather
  than being duplicated per leaf — one glossary and one extension
  procedure serve all 5 CLIs since they share the same entity taxonomy
  (`cli_doc_des.rulebook.md`).
