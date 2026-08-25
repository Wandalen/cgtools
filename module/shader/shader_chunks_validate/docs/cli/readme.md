# shader_chunks_validate CLI Documentation

Command reference for the `validate` command of the `shader_chunks`
terminal tool — registry-wide integrity linting over every WGSL shader
chunk bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md). The five
checks (manifest drift, duplicate names, missing/cyclic dependencies,
naga WGSL compilation) live in
[`shader_chunks_validate_core`](../../../shader_chunks_validate_core/readme.md);
this crate wires them to the CLI via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
`validate` is the only command in the family that takes no target at
all — every other command selects one chunk, one file, or an explicit
chunk-name list; `validate` always runs over the whole compiled-in
registry — see
[`command_group/01_validate.md`](command_group/01_validate.md#why-not-merge-into-query-compose-preview-render)
for why it stays its own group instead of folding into an existing one.
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

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for this
crate's 1-command slice (`validate`) of the `shader_chunks` family; no
incomplete-content placeholders. This crate declares no `param/`,
`param_group/`, or `type/` of its own — `validate` takes zero
parameters. Its output is unstructured text, reusing the
[`plain_text`](../../../shader_chunks_compose/docs/cli/format/01_plain_text.md)
format already owned by
[`shader_chunks_compose`](../../../shader_chunks_compose/docs/cli/readme.md)
rather than declaring a duplicate `format/` entity.
**Implementation Status:** Matches shipped code — `src/lib.rs` (CLI
wiring) and `src/bin/shader_chunks_validate.rs`; engine in
[`shader_chunks_validate_core`](../../../shader_chunks_validate_core/readme.md);
tests in `tests/validate_cli_test.rs`. A right-sized `tests/docs/cli/`
mirror exists
([`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md))
covering the command/command_group tiers.

## Navigation

- [`command/`](command/readme.md) — the 1 command (`validate`)
- [`command_group/`](command_group/readme.md) — the 1 command group (`Validate`)
- [`plain_text` format](../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) — output format (owned by `shader_chunks_compose`)
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
  procedure serve all 6 CLIs since they share the same entity taxonomy
  (`cli_doc_des.rulebook.md`).
- **`param/`, `param_group/`, `type/`, `format/` omitted here.**
  `validate` declares zero parameters and reuses one output format
  (`plain_text`), already fully documented as a
  [`shader_chunks_compose`](../../../shader_chunks_compose/readme.md)
  entity (since `compose`/`preview`/`render` already produce unstructured
  text through the same rendering path) — duplicating that file here
  would violate leaf-locality's own no-duplication corollary. This crate
  introduces no new type.
