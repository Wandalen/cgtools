# shader_chunks_params CLI Documentation

Command reference for the `tunables` command of the `shader_chunks`
terminal tool — listing the declared tunable parameters (`//@ param:`
lines) of a WGSL shader chunk bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md). The
declaration-parsing and range-inference engine lives in
[`shader_chunks_params_core`](../../../shader_chunks_params_core/readme.md);
this crate wires it to the CLI via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
`tunables` is the only command in the family whose data source is a
second crate rather than `shader_chunks_core::CHUNKS` alone — see
[`command_group/01_parameters.md`](command_group/01_parameters.md#why-not-merge-into-query)
for why it stays its own group instead of folding into Query. This
crate is one of 6 leaf CLIs assembled by the
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
crate's 1-command slice (`tunables`) of the `shader_chunks` family; no
incomplete-content placeholders. This crate declares no `param/`,
`param_group/`, `type/`, or `format/` of its own — `tunables`' sole
parameter (`name`) and output format (`table_plain`) are owned by
[`shader_chunks_query`](../../../shader_chunks_query/docs/cli/readme.md),
referenced rather than duplicated.
**Implementation Status:** Matches shipped code — `src/lib.rs` (CLI
wiring) and `src/bin/shader_chunks_params.rs`; engine in
[`shader_chunks_params_core`](../../../shader_chunks_params_core/readme.md);
tests in `tests/tunables_test.rs`. A right-sized `tests/docs/cli/`
mirror exists
([`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md))
covering the command/command_group tiers.

## Navigation

- [`command/`](command/readme.md) — the 1 command (`tunables`)
- [`command_group/`](command_group/readme.md) — the 1 command group (`Parameters`)
- [`name` param](../../../shader_chunks_query/docs/cli/param/01_name.md) — sole parameter (owned by `shader_chunks_query`)
- [`table_plain` format](../../../shader_chunks_query/docs/cli/format/01_table_plain.md) — output format (owned by `shader_chunks_query`)
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
- **`param/`, `param_group/`, `type/`, `format/` omitted here.**
  `tunables` declares exactly one parameter (`name`) and reuses one
  output format (`table_plain`), both already fully documented as
  `shader_chunks_query` entities (since `.list`/`.get`/`.tree` accept
  `name` too, and `.list`/`.get`/`.tags` produce `table_plain`) —
  duplicating either file here would violate leaf-locality's own
  no-duplication corollary. This crate introduces no new type.
