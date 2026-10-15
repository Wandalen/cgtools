# Command Group :: 7. Validate

### Pattern

Registry-wide integrity linting: run every check across the *whole*
bundled set in one pass and report every finding, rather than operating
on a caller-selected chunk, file, or explicit chunk-name list the way
every other group in this family does.

### Purpose

Let a shader author (or CI) catch structural registry problems — a stale
generated manifest, a name collision, a broken dependency edge, a cycle,
or WGSL that doesn't actually compile — in one command, before any of
them surfaces later as a confusing downstream error from `compose`,
`preview`, or `render`.

### Semantic Coherence Test

"The member command answers a registry-health-shaped question: is the
whole compiled-in chunk set internally consistent, and does everything in
it actually compile?" `.validate` is the only command in the family whose
subject is the registry's own consistency rather than its content — every
other command (including the zero-parameter `.tags`) still answers a
question about specific chunks or chunk records.

### Why NOT Merge Into Query, Compose, Preview, Render

The [Query](../../../../shader_chunks_query/docs/cli/command_group/01_query.md)
group's own stated invariant is that only `shader_chunks_core::CHUNKS` is
consulted — true of `validate` too, so registry-only access alone does not
exclude it. What actually distinguishes `validate` is its output *type*
and *scope*: Query's commands (including the zero-parameter `.tags`)
render chunk records or record-derived summaries; `validate` renders
[`Finding`](../../../../shader_chunks_validate_core/readme.md)
diagnostics, an entirely different shape none of Query's `fields::`/
`sort::`/`tag::`/etc. parameters could apply to. Against
[Compose](../../../../shader_chunks_compose/docs/cli/command_group/01_compose.md)/
[Preview](../../../../shader_chunks_preview/docs/cli/command_group/01_preview.md)/
[Render](../../../../shader_chunks_render/docs/cli/command_group/01_render.md):
those three also naga-validate WGSL, but only as a side effect of
building one artifact from a caller-selected target — raw composed text,
a served preview bundle, or a rendered PNG. `validate` is the only
command whose primary purpose is validation itself, run across *every*
chunk's own transitive closure, producing no artifact at all — no file
written, no server started, no WGSL printed — only a diagnostic report.

### Invariants

- Idempotent: identical input (the registry) always produces identical
  output.
- No side effects outside stdout content and process exit code — no file
  is ever written, unlike `compose out::`/`preview`/`render`.
- Every check runs over the full input set in one pass; no check
  short-circuits or gets skipped because another check already found a
  related problem in the same chunk (verified by the two
  no-derivative-duplicate tests below).
- Zero findings renders an explicit all-clear message
  (`"registry is clean: {n} chunks, 0 findings"`), never blank output.
- Takes zero arguments — one of two zero-parameter commands in the
  family (`.tags` is the other), and the only one whose subject is the
  registry's own internal consistency rather than its content.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.validate`](../command/01_validate.md) | Member — registry-wide integrity report |

**Membership:** 1 of the 9 commands across the `shader_chunks` family; the
full partition across all 7 command groups (spanning all 6 leaf CLIs) is
stated in [the family index](../../../../shader_chunks/docs/cli/readme.md).
A single-member group is deliberate — the boundary is question shape and
output type, not command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/01_validate.md`](../../../tests/docs/cli/command_group/01_validate.md) | Group-level test specification |
| [`../../../../shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs`](../../../../shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs) | The 5 checks' own fixture-isolated behavior, including the two no-derivative-duplicate invariants and the real-bundled-registry sanity check |

### Typical Patterns

Run `validate` before `compose`/`preview`/`render` against a freshly
edited or newly added chunk — a registry-wide problem (a typo'd
dependency name, a manifest left out of sync with its own WGSL text)
surfaces here as a labeled finding instead of a harder-to-diagnose panic
or naga error several commands downstream.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
