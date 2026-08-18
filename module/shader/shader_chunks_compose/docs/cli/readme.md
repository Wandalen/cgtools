# shader_chunks_compose CLI Documentation

Command/format reference for the `compose` command of the
`shader_chunks` terminal tool — previewing the dependency-ordered WGSL
text produced by composing one or more chunks bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md). The
engine lives directly in this crate's `src/lib.rs` (no separate `_core`
crate); it wires to the CLI via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
`compose` accepts `names` (positional) and `transitive` (closure
switch), both owned and documented by
[`shader_chunks_query`](../../../shader_chunks_query/docs/cli/readme.md),
plus this crate's own `out` — write the composed WGSL to a file instead
of printing it to stdout.
This crate is one of 6 leaf CLIs assembled by the
[`shader_chunks`](../../../shader_chunks/docs/cli/readme.md) aggregator
— see that readme for the family-wide 9-command list and Scope
Decisions.

## Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | — | — | Complete |
| command/ | ✅ | ✅ | ✅ | — | — | Complete |
| command_group/ | ✅ | ✅ | ✅ | — | — | Complete |
| format/ | ✅ | ✅ | ✅ | — | — | Complete |
| param/ | ✅ | ✅ | ✅ | — | — | Complete |

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for this
crate's 1-command slice (`compose`) of the `shader_chunks` family; no
incomplete-content placeholders. This crate declares no `param_group/`
or `type/` of its own — `out` is a plain String with no shared group
membership. `names`/`transitive` remain owned by `shader_chunks_query`;
`out` is this crate's own first owned parameter.
**Implementation Status:** Matches shipped code — `src/lib.rs` (engine +
CLI wiring, contributes `help_groups()`/`help_examples()`/`commands()`
to the aggregator), tested by
`tests/shader_chunks_compose_test.rs`. A right-sized `tests/docs/cli/`
mirror exists
([`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md))
covering the command/command_group tiers.

## Navigation

- [`command/`](command/readme.md) — the 1 command (`compose`)
- [`command_group/`](command_group/readme.md) — the 1 command group (`Compose`)
- [`format/`](format/readme.md) — the 1 output format this crate introduces (`plain_text`)
- [`param/`](param/readme.md) — the 1 parameter this crate introduces (`out`)
- [`shader_chunks_query/docs/cli/param/readme.md`](../../../shader_chunks_query/docs/cli/param/readme.md) — `names`/`transitive` parameter definitions (owned by `shader_chunks_query`)
- [`../../../shader_chunks/docs/cli/dictionary.md`](../../../shader_chunks/docs/cli/dictionary.md) — family-wide domain term glossary
- [`../../../shader_chunks/docs/cli/procedure.md`](../../../shader_chunks/docs/cli/procedure.md) — how to extend a `docs/cli/` tree when an entity is added or removed
- [`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md) — test specification mirror
- [`../../readme.md`](../../readme.md) — crate readme (purpose, examples, links back here)
- [`../../../shader_chunks/docs/cli/readme.md`](../../../shader_chunks/docs/cli/readme.md) — family index (all 6 leaf CLIs, all 9 commands)

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
- **`param_group/`, `type/` omitted here.** `out` is a plain String
  parameter belonging to no parameter group (an artifact-path selector,
  like `render`'s own `out`) and needs no dedicated type beyond the
  built-in `String` — nothing to declare in either collection.
  `names`/`transitive`, `compose`'s other two parameters, remain
  declared and documented by `shader_chunks_query` since `list`/`get`
  share them verbatim — duplicating those definitions here would
  violate leaf-locality's own no-duplication counterpart; this crate
  references them instead. `out` itself IS declared here
  ([`param/`](param/readme.md)) since it belongs to no other crate.
