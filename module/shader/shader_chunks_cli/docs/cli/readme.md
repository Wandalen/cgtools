# shader_chunks_cli CLI Documentation

Command/parameter/type reference for the `shader_chunks_cli` terminal tool —
5 read-only commands for listing, inspecting, and composing the WGSL shader
chunks bundled by [`shader_chunks`](../../../shader_chunks/readme.md).

## Completion Matrix

| File | L1 | L2 | L3 | L4 | L5 | Status |
|------|----|----|----|----|----|--------|
| readme.md | ✅ | ✅ | ✅ | — | — | Complete |
| command/ | ✅ | ✅ | ✅ | — | — | Complete |
| param/ | ✅ | ✅ | ✅ | — | — | Complete |
| type/ | ✅ | ✅ | ✅ | — | — | Complete |
| format/ | ✅ | ✅ | ✅ | — | — | Complete |
| command_group.md | ✅ | ✅ | ✅ | — | — | Complete |
| dictionary.md | — | ✅ | ✅ | — | — | Complete |
| procedure.md | — | — | — | — | — | Complete (directory infrastructure, not a leveled entity) |

**Current Level:** L3 (Specification Complete)
**Design Completeness:** All required L1-L3 entities present for a 5-command, single-domain-object CLI; zero TBD markers; every command/param/type/format instance has full required content and cross-references.
**Implementation Status:** Matches shipped code — `src/main.rs` (dispatch), `src/lib.rs` (command logic), `tests/` (direct-call + subprocess coverage). A right-sized `tests/docs/cli/` mirror exists (`../../tests/docs/cli/readme.md`) covering param/command/command_group/type tiers; it is not a full L4/L5 exhaustive test-specification build-out (not required by the governing task).

## Navigation

- [`command/`](command/readme.md) — the 5 commands (`list`, `get`, `tags`, `tree`, `compose`)
- [`param/`](param/readme.md) — the 2 parameters (`name`, `names`)
- [`type/`](type/readme.md) — the 1 domain type (`ChunkName`)
- [`format/`](format/readme.md) — the 3 output formats (table, tree, plain text)
- [`command_group.md`](command_group.md) — the 1 command group (`Inspection`)
- [`dictionary.md`](dictionary.md) — domain term glossary
- [`procedure.md`](procedure.md) — how to extend this directory when a command/param/type/format is added or removed
- [`../../tests/docs/cli/readme.md`](../../tests/docs/cli/readme.md) — test specification mirror
- [`../../readme.md`](../../readme.md) — crate readme (purpose, examples, links back here)

## Scope Decisions

Deliberate, disclosed deviations from a literal maximal reading of
`cli_doc_des.rulebook.md`, made because this CLI is 5 read-only inspection
commands over one static manifest — not because the rulebook was misread:

- **`user_story/` omitted.** The rulebook requires a `### Referenced User
  Stories` section (≥1 row) on every `command`/`param`/`format`/
  `command_group` instance, but creating the collection itself needs enough
  distinct narratives to be meaningful — this CLI's whole surface is "a
  developer inspects/composes bundled shader chunks from the terminal," one
  story stretched thin across 5 commands, not 5 genuine personas/goals.
  Below the rulebook's own 5-instance guidance for the collection to be
  worthwhile. Every file that formally requires the section still carries
  it, pointing back to this paragraph instead of a fabricated story.
- **`command_noun`/`command_verb` omitted.** Both require ≥3 domain nouns;
  this CLI operates on exactly one domain object ("chunk").
- **`param_group/` omitted.** No two commands share a *set* of ≥2
  co-occurring parameters (`§ Parameters Documentation : Parameter Groups`'s
  own worked example groups `spreadsheet::`+`sheet::` — two parameters used
  together; nothing here co-occurs, since each command takes at most one
  identifying parameter).
- **`env_param.md`/`config_param.md` omitted.** `shader_chunks_cli` reads no
  environment variables and no config file — argv only
  (`std::env::args()`, confirmed via `grep -n "std::env" src/main.rs`) — so
  both mechanism-documentation files would describe zero mechanisms.
- **`index.md` omitted.** Not a canonical entity anywhere in
  `cli_doc_des.rulebook.md` (confirmed by exhaustive grep across the
  rulebook) — this file's own Navigation section above serves the index
  role the governing task's filing anticipated.
- **`command_group.md` included** even though not named in the governing
  task's `## In Scope` list — the rulebook's L3 Blocker Conditions require
  it unconditionally once ≥3 commands exist (this CLI has 5), and the
  task's own Acceptance Criterion C9 gates on rulebook compliance, not the
  literal In Scope enumeration.
