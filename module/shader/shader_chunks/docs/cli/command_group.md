# Command Group :: Inspection

### Pattern

All 5 commands (`.list`, `.get`, `.tags`, `.tree`, `.compose`) are read-only
inspections over the compiled-in chunk registry
(`shader_chunks_core::ALL_CHUNKS`). None mutate state, none write files, none
touch the network or filesystem beyond the WGSL text already embedded via
`include_str!` at compile time.

### Purpose

Let a shader author explore what chunks exist, inspect one chunk's full
detail, discover tags, view a chunk's dependency tree (or the whole
forest), and preview composed WGSL output — before wiring
`shader_chunks_core::try_compose` into a real render pipeline.

### Semantic Coherence Test

"All 5 commands answer a read-only question about the compiled-in chunk
registry." `.list` answers "what chunks exist"; `.get` answers "what does
this one chunk look like"; `.tags` answers "what tags exist and on which
chunks"; `.tree` answers "what does this chunk depend on"; `.compose`
answers "what would composing these chunks produce." Every command fits
the single sentence — none require a second group.

### Why NOT Split Into Two Groups

A plausible split is "browsing" (`list`/`get`/`tags`/`tree`) vs.
"composing" (`compose`). Rejected: `.compose`'s output is itself only a
preview of composed WGSL text printed to stdout — it never writes a file
or registers a new chunk, so it carries the exact same invariants (Yes)
below as the other 4 commands. Splitting would draw a group boundary with
no invariant actually differing across it, which the Semantic Coherence
Test above already answers as one group.

### Invariants

- Idempotent: identical input always produces identical output.
- No side effects outside stdout content and process exit code.
- Every command operates only on `shader_chunks_core::ALL_CHUNKS` — no
  runtime-discovered, user-supplied, or filesystem-loaded chunk source
  exists in this CLI.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.list`](command/01_list.md) | Member — enumerate all chunks |
| 2 | [`.get`](command/02_get.md) | Member — inspect one chunk |
| 3 | [`.tags`](command/03_tags.md) | Member — enumerate tags |
| 4 | [`.tree`](command/04_tree.md) | Member — show dependency tree |
| 5 | [`.compose`](command/05_compose.md) | Member — preview composed output |

**Total/complete partition:** all 5 commands this CLI exposes belong to
this one group — there is no command outside it.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../tests/docs/cli/command_group/01_inspection.md`](../../tests/docs/cli/command_group/01_inspection.md) | Group-level test specification |
| [`../../tests/cli_subprocess_test.rs`](../../tests/cli_subprocess_test.rs) | Subprocess assertions exercising all 5 commands |

### Typical Patterns

Explore then compose: run `.list` or `.tags` to discover candidate chunks,
`.get` or `.tree` to inspect one in detail, then `.compose` to preview the
final WGSL before integrating it into a real pipeline.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`readme.md` § Scope
Decisions](readme.md#scope-decisions).)*
