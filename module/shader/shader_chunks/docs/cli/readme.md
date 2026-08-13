# shader_chunks CLI Documentation

Command/parameter/type reference for the `shader_chunks` terminal tool —
6 read-only commands for querying, inspecting, composing, and
introspecting the tunable parameters of the WGSL shader chunks bundled by
[`shader_chunks_core`](../../../shader_chunks_core/readme.md). The two
query commands (`list`, `get`) share one engine with a 20-parameter
filter/projection/formatting surface; `tree` renders the dependency
graph; `compose` previews composed WGSL; `tunables` lists a chunk's
declared tunable parameters via
[`shader_chunks_params`](../../../shader_chunks_params/readme.md).

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
| dictionary.md | — | ✅ | ✅ | — | — | Complete |
| procedure.md | — | — | — | — | — | Complete (directory infrastructure, not a leveled entity) |

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for a 6-command, single-domain-object CLI; no incomplete-content placeholders; every command/command_group/param/param_group/type/format instance has full required content and cross-references.
**Implementation Status:** Matches shipped code — `src/cli.rs` (dispatch, argument surface, grouped help), `src/lib.rs` (query engine + command logic), `tests/` (direct-call + subprocess coverage). A right-sized `tests/docs/cli/` mirror exists (`../../tests/docs/cli/readme.md`) covering param/param_group/command/command_group/type tiers; it is not a full L4/L5 exhaustive test-specification build-out (not required by the governing task).

## Navigation

- [`command/`](command/readme.md) — the 6 commands (`list`, `get`, `tags`, `tree`, `compose`, `tunables`)
- [`command_group/`](command_group/readme.md) — the 4 command groups (`Query`, `Graph`, `Compose`, `Parameters`)
- [`param/`](param/readme.md) — the 21 parameters (2 positional selectors + 19 shared named query parameters)
- [`param_group/`](param_group/readme.md) — the 3 parameter groups (`filtering`, `projection`, `formatting`)
- [`type/`](type/readme.md) — the 10 semantic types (`ChunkName`, the query enums, selectors, `Switch`, `NonNegativeInteger`)
- [`format/`](format/readme.md) — the 8 output formats (6 selectable via `format::`, plus tree and WGSL text)
- [`dictionary.md`](dictionary.md) — domain term glossary
- [`procedure.md`](procedure.md) — how to extend this directory when an entity is added or removed
- [`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md) — test specification mirror
- [`../../readme.md`](../../readme.md) — crate readme (purpose, examples, links back here)

## Scope Decisions

Deliberate, disclosed deviations from a literal maximal reading of
`cli_doc_des.rulebook.md`, made because this CLI is 6 read-only inspection
commands over one static manifest — not because the rulebook was misread:

- **`user_story/` omitted.** The rulebook requires a `### Referenced User
  Stories` section (≥1 row) on every `command`/`param`/`format`/
  `command_group` instance, but creating the collection itself needs enough
  distinct narratives to be meaningful — this CLI's whole surface is "a
  developer inspects/composes bundled shader chunks from the terminal," one
  story stretched thin across 6 commands, not 6 genuine personas/goals.
  Below the rulebook's own 5-instance guidance for the collection to be
  worthwhile. Every file that formally requires the section still carries
  it, pointing back to this paragraph instead of a fabricated story.
- **`command_noun`/`command_verb` omitted.** Both require ≥3 domain nouns;
  this CLI operates on exactly one domain object ("chunk").
- **`env_param.md`/`config_param.md` omitted.** `shader_chunks` reads no
  environment variables and no config file — argv only
  (`std::env::args()`, confirmed via `grep -n "std::env" src/cli.rs`) — so
  both mechanism-documentation files would describe zero mechanisms.
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
  exist (this CLI has 5).
