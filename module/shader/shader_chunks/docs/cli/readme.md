# shader_chunks CLI Documentation

Command/parameter/type reference for the `shader_chunks` terminal tool —
8 commands for querying, inspecting, composing, previewing, rendering,
and introspecting the tunable parameters of the WGSL shader chunks
bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md).
`shader_chunks` itself is a thin aggregator: each command group's engine
and CLI wiring lives in its own crate
([`shader_chunks_query_core`](../../../shader_chunks_query_core/readme.md)+[`shader_chunks_query`](../../../shader_chunks_query/readme.md),
[`shader_chunks_compose`](../../../shader_chunks_compose/readme.md),
[`shader_chunks_params_core`](../../../shader_chunks_params_core/readme.md)+[`shader_chunks_params`](../../../shader_chunks_params/readme.md),
[`shader_chunks_preview_core`](../../../shader_chunks_preview_core/readme.md)+[`shader_chunks_preview`](../../../shader_chunks_preview/readme.md),
[`shader_chunks_render_core`](../../../shader_chunks_render_core/readme.md)+[`shader_chunks_render`](../../../shader_chunks_render/readme.md)),
sharing argument/dispatch plumbing via
[`shader_chunks_cli_core`](../../../shader_chunks_cli_core/readme.md).
The two query commands (`list`, `get`) share one engine with a
20-parameter filter/projection/formatting surface; `tree` renders the
dependency graph; `compose` previews composed WGSL; `tunables` lists a
chunk's declared tunable parameters via
[`shader_chunks_params`](../../../shader_chunks_params/readme.md);
`preview` builds and naga-validates a live browser preview bundle,
rendered by the wasm-only
[`shader_chunks_preview_web`](../../../shader_chunks_preview_web/readme.md)
runner; `render` writes one headless-GPU frame of the same bundle as a
static PNG.

## Family Completion Matrix

This aggregator is a thin family index — the 6 entity-class collections
(`command/`, `command_group/`, `param/`, `param_group/`, `type/`,
`format/`) that used to live directly under this directory have moved
into the crate that actually implements each command group (see Scope
Decisions below). What remains local to this readme is the family-wide
lookup surface; each leaf owns its own full L1-L3 documentation tree.

| Leaf crate | Commands | Groups | Own Params | Own Types | Own Formats | Docs |
|------------|----------|--------|------------|-----------|-------------|------|
| [`shader_chunks_query`](../../../shader_chunks_query/docs/cli/readme.md) | `list`, `get`, `tags`, `tree` | Query, Graph | 21 | 10 | 7 | Complete |
| [`shader_chunks_compose`](../../../shader_chunks_compose/docs/cli/readme.md) | `compose` | Compose | 0 (reuses `names`/`transitive`) | 0 | 1 | Complete |
| [`shader_chunks_params`](../../../shader_chunks_params/docs/cli/readme.md) | `tunables` | Parameters | 0 (reuses `name`) | 0 | 0 (reuses `table_plain`) | Complete |
| [`shader_chunks_preview`](../../../shader_chunks_preview/docs/cli/readme.md) | `preview` | Preview | 2 | 0 | 0 (reuses `plain_text`) | Complete |
| [`shader_chunks_render`](../../../shader_chunks_render/docs/cli/readme.md) | `render` | Render | 3 | 1 | 0 (reuses `plain_text`) | Complete |
| **Total** | **8** | **6** | **26** | **11** | **8** | |

| Local file | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | — | — | Complete |
| dictionary.md | — | ✅ | ✅ | — | — | Complete |
| procedure.md | — | — | — | — | — | Complete (directory infrastructure, not a leveled entity) |

**Current Level:** L3 (Specification Complete), family-wide.
**Design Completeness:** All required L1-L3 entities present across the
5 leaf crates for this 8-command, single-domain-object CLI; no
incomplete-content placeholders; every command/command_group/param/
param_group/type/format instance has full required content and
cross-references, verified by a repo-wide link-integrity sweep.
**Implementation Status:** Matches shipped code across the crate family
— `shader_chunks/src/lib.rs` (thin aggregator: concatenates each
utility's `help_groups()`/`help_examples()`/`commands()` and calls
`shader_chunks_cli_core::run()`), `shader_chunks_cli_core/src/lib.rs`
(dispatch, argument surface, grouped help — the former `src/cli.rs`),
and each utility's own engine crate plus CLI-wiring crate
(`shader_chunks_query_core`+`shader_chunks_query`, `shader_chunks_compose`
— no separate `_core`, see its own readme — `shader_chunks_params_core`+
`shader_chunks_params`, `shader_chunks_preview_core`+
`shader_chunks_preview`, `shader_chunks_render_core`+
`shader_chunks_render`), each with its own `tests/`. Each leaf's own
`tests/docs/cli/` mirror (linked from its readme above) covers its slice
of the param/param_group/command/command_group/type tiers; none is a
full L4/L5 exhaustive test-specification build-out (not required by the
governing task).

## Navigation

- [`shader_chunks_query/docs/cli/`](../../../shader_chunks_query/docs/cli/readme.md) — `list`, `get`, `tags`, `tree`; the shared query engine's params/param_groups/types/formats
- [`shader_chunks_compose/docs/cli/`](../../../shader_chunks_compose/docs/cli/readme.md) — `compose`; the `plain_text` format
- [`shader_chunks_params/docs/cli/`](../../../shader_chunks_params/docs/cli/readme.md) — `tunables`
- [`shader_chunks_preview/docs/cli/`](../../../shader_chunks_preview/docs/cli/readme.md) — `preview`; `file`/`serve` params
- [`shader_chunks_render/docs/cli/`](../../../shader_chunks_render/docs/cli/readme.md) — `render`; `out`/`size`/`time` params, the `Float` type
- [`dictionary.md`](dictionary.md) — family-wide domain term glossary (stays centralized — see Scope Decisions)
- [`procedure.md`](procedure.md) — how to extend a `docs/cli/` tree when an entity is added or removed (stays centralized — see Scope Decisions)
- [`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md) — test specification family index
- [`../../readme.md`](../../readme.md) — crate readme (purpose, examples, links back here)

## Scope Decisions

Deliberate, disclosed deviations from a literal maximal reading of
`cli_doc_des.rulebook.md`, made because this CLI is 8 inspection and
artifact-producing commands over one static manifest — not because the
rulebook was misread:

- **`user_story/` omitted.** The rulebook requires a `### Referenced User
  Stories` section (≥1 row) on every `command`/`param`/`format`/
  `command_group` instance, but creating the collection itself needs enough
  distinct narratives to be meaningful — this CLI's whole surface is "a
  developer inspects/composes/previews/renders bundled shader chunks from
  the terminal," one story stretched thin across 8 commands, not 8 genuine
  personas/goals. Below the rulebook's own 5-instance guidance for the
  collection to be worthwhile. Every file that formally requires the
  section still carries it, pointing back to this paragraph instead of a
  fabricated story.
- **`command_noun`/`command_verb` omitted.** Both require ≥3 domain nouns;
  this CLI operates on exactly one domain object ("chunk").
- **`env_param.md`/`config_param.md` omitted.** `shader_chunks` reads no
  environment variables and no config file at runtime — argv only
  (`std::env::args()` in `shader_chunks_cli_core/src/lib.rs`, confirmed
  via `grep -rn "std::env" module/shader/shader_chunks*/src`) — so both
  mechanism-documentation files would describe zero runtime mechanisms.
  `shader_chunks_preview` resolves two paths via compile-time
  `env!("CARGO_MANIFEST_DIR")`; that is a build-time constant baked into
  the binary, not a runtime environment read, so it does not reopen this
  Scope Decision.
- **`index.md` omitted.** Not a canonical entity anywhere in
  `cli_doc_des.rulebook.md` (confirmed by exhaustive grep across the
  rulebook) — this file's own Navigation section above serves the index
  role the governing task's filing anticipated.
- **`param_group/` included** (formerly omitted). The omission rationale —
  "no two commands share a set of ≥2 co-occurring parameters" — became
  false when `list` and `get` unified behind one query engine: they now
  share 19 named parameters verbatim, partitioned into 3 groups.
- **`command_group/` is a directory** with one dedicated file per group,
  never a flat `command_group.md` — one file cannot carry per-group
  coherence tests, invariants, and membership tables once more than one
  group exists, and the collection is directory-form from the start so
  adding a group never restructures it. The rulebook's L3 Blocker
  Conditions require the collection unconditionally once ≥3 commands
  exist (this CLI has 8).
- **`command/`, `command_group/`, `param/`, `param_group/`, `type/`, and
  `format/` moved out of this aggregator entirely**, relocated one level
  into whichever leaf crate actually implements each entity
  (`shader_chunks_query`, `_compose`, `_params`, `_preview`, `_render` —
  see the Family Completion Matrix and Navigation above). This aggregator
  crate contains no command logic of its own — every command, parameter,
  type, and format is owned and tested inside its implementing crate —
  so documenting those entities here would duplicate content that
  already lives at its point of implementation, violating leaf-locality
  (docs, tests, and code belong as close to the leaves of the dependency
  tree as possible). Only `readme.md`, `dictionary.md` (shared glossary),
  and `procedure.md` (shared extension procedure) stay centralized, since
  those three serve the whole family rather than any single entity.
