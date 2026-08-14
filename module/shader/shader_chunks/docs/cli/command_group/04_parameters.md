# Command Group :: 4. Parameters

### Pattern

Tunable-surface introspection: parse a chunk's `//@ param:` manifest
lines via `shader_chunks_params::chunk_discover` and print the parsed
result — one row per declared parameter, kind/type/range/source — rather
than a filtered set of chunk records or the dependency graph itself.

### Purpose

Let a shader author (or a UI/tooling layer — see
`examples/minwebgpu/shader_chunk_preview/readme.md` for a live browser
consumer) discover which values on a chunk are meant to be tuned — and by
what range — without reading the chunk's WGSL source directly.

### Semantic Coherence Test

"The member command answers a tunability-shaped question about one
chunk — which of its values are parameters, and what range each one
takes." `.tunables` is the only command whose data source is
`shader_chunks_params` rather than `shader_chunks_core::CHUNKS` alone.

### Why NOT Merge Into Query

The [Query](01_query.md) group's own stated invariant is that only
`shader_chunks_core::CHUNKS` is consulted — no filesystem, environment,
or network access beyond the compiled-in registry. `.tunables` breaks
that invariant by construction: it calls
`shader_chunks_params::chunk_discover`, a second crate that parses
`//@ param:` lines out of the chunk's own WGSL text via its own grammar,
independent of any of Query's 19 filter/projection/formatting
parameters. Its output rows are parameters, not chunk records — none of
`fields::`, `sort::`, `tag::`, etc. apply. Merging would put a
single-crate-dependency command inside a group whose entire contract is
"the registry alone is enough."

### Invariants

- Idempotent: identical input always produces identical output.
- No side effects outside stdout content and process exit code.
- `shader_chunks_core::CHUNKS` (for chunk lookup) and the resolved
  chunk's own `wgsl` field (for parameter discovery, via
  `shader_chunks_params::chunk_discover`) are consulted — no filesystem,
  environment, or network access beyond what's already compiled in.
- A chunk with zero declared `//@ param:` lines renders an explicit
  message, never a blank table or a false error.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.tunables`](../command/06_tunables.md) | Member — declared tunable parameters for one chunk |

**Membership:** 1 of the 6 commands; the partition across all groups is
stated in [`readme.md`](readme.md). A single-member group is deliberate —
the boundary is data-source (a second crate, `shader_chunks_params`), not
command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/04_parameters.md`](../../../tests/docs/cli/command_group/04_parameters.md) | Group-level test specification |
| [`../../../tests/shader_chunks_test.rs`](../../../tests/shader_chunks_test.rs) | `tunables_of_chunk_lists_declared_and_inferred_parameters`, `tunables_zero_declared_params_reports_explicit_message_not_blank_or_error`, `tunables_unknown_chunk_reports_unknown_chunk_error` |

### Typical Patterns

Inspect with [Query](01_query.md) or [Graph](02_graph.md) to find a
chunk of interest, then `tunables <name>` to see what on it is meant to
be tuned before wiring a UI slider or a compile-time override to it —
`examples/minwebgpu/shader_chunk_preview/` is exactly this workflow
carried through to a live browser UI.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
