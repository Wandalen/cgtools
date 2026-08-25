# shader_chunks_preview CLI Documentation

Command/parameter reference for the `preview` command of the
`shader_chunks` terminal tool — building and naga-validating a live
browser preview bundle for one WGSL shader chunk bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md). The
engine lives in
[`shader_chunks_preview_core`](../../../shader_chunks_preview_core/readme.md);
this crate wires it to the CLI via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
`preview` writes the composed bundle into the wasm-only
[`shader_chunks_preview_web`](../../../shader_chunks_preview_web/readme.md)
runner crate and, by default, hands off to the browser via the repo's
shared `action/browser_serve` script; `serve::0` stops after writing and
prints a summary instead. This crate is one of 6 leaf CLIs assembled by
the [`shader_chunks`](../../../shader_chunks/docs/cli/readme.md)
aggregator — see that readme for the family-wide 9-command list and
Scope Decisions.

## Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | — | — | Complete |
| command/ | ✅ | ✅ | ✅ | — | — | Complete |
| command_group/ | ✅ | ✅ | ✅ | — | — | Complete |
| param/ | ✅ | ✅ | ✅ | — | — | Complete |

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for this
crate's 1-command slice (`preview`) of the `shader_chunks` family; no
incomplete-content placeholders; every command/command_group/param
instance has full required content and cross-references. This crate
declares no `param_group/`, `type/`, or `format/` collection — `preview`
introduces no new semantic type (`file` reuses plain `String`, `serve`
reuses [`Switch`](../../../shader_chunks_query/docs/cli/type/07_switch.md))
and no new output format (its summary line reuses
[`plain_text`](../../../shader_chunks_compose/docs/cli/format/01_plain_text.md)),
and its 2 own parameters never co-occur as a filter/projection/formatting
set, so no group applies.
**Implementation Status:** Matches shipped code — `src/lib.rs` (CLI
wiring, contributes `help_groups()`/`help_examples()`/`commands()` to
the aggregator); engine in
[`shader_chunks_preview_core`](../../../shader_chunks_preview_core/readme.md);
subprocess tests in `tests/preview_cli_test.rs`. A right-sized
`tests/docs/cli/` mirror exists
([`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md))
covering the param/command/command_group tiers.

## Navigation

- [`command/`](command/readme.md) — the 1 command (`preview`)
- [`command_group/`](command_group/readme.md) — the 1 command group (`Preview`)
- [`param/`](param/readme.md) — the 2 own parameters (`file`, `serve`); `preview` also accepts `name` from [`shader_chunks_query`](../../../shader_chunks_query/docs/cli/param/01_name.md)
- [`../../../shader_chunks/docs/cli/dictionary.md`](../../../shader_chunks/docs/cli/dictionary.md) — family-wide domain term glossary
- [`../../../shader_chunks/docs/cli/procedure.md`](../../../shader_chunks/docs/cli/procedure.md) — how to extend a `docs/cli/` tree when an entity is added or removed
- [`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md) — test specification mirror
- [`../../readme.md`](../../readme.md) — crate readme (purpose, examples, links back here)
- [`../../../shader_chunks/docs/cli/readme.md`](../../../shader_chunks/docs/cli/readme.md) — family index (all 6 leaf CLIs, all 9 commands)

## Scope Decisions

This crate inherits the family-wide Scope Decisions stated in
[the family index](../../../shader_chunks/docs/cli/readme.md#scope-decisions)
(`user_story/`, `command_noun`/`command_verb`, `env_param.md`/
`config_param.md`, and `index.md` omissions — including the note that
this crate's two compile-time `env!("CARGO_MANIFEST_DIR")` path
resolutions are a build-time constant, not a runtime environment read,
so they do not reopen the `env_param.md` decision) — none of those
rationales are specific to this crate, so they are not restated here.
One crate-local addendum:

- **`dictionary.md`/`procedure.md` omitted here.** Both stay centralized
  at the family index
  ([`dictionary.md`](../../../shader_chunks/docs/cli/dictionary.md),
  [`procedure.md`](../../../shader_chunks/docs/cli/procedure.md)) rather
  than being duplicated per leaf — one glossary and one extension
  procedure serve all 5 CLIs since they share the same entity taxonomy
  (`cli_doc_des.rulebook.md`).
