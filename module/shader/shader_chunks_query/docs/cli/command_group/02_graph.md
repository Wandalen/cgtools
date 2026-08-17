# Command Group :: 2. Graph

### Pattern

Relationship rendering: walk the `depends_on` edges of the compiled-in
chunk registry and print the structure itself — parent-before-child,
indented, tags trailing — rather than a filtered set of rows.

### Purpose

Let a shader author see how chunks relate: one chunk's full dependency
chain (`tree fbm3`), or the whole forest of root chunks (`tree` with no
argument).

### Semantic Coherence Test

"The member command answers a relationship-shaped question about the
compiled-in chunk registry — what depends on what, rendered as the graph
itself." `.tree` is the only command whose output rows are graph *edges*
(nesting), not chunk *records*.

### Why NOT Merge Into Query

The [Query](01_query.md) group's `depends_on::`/`transitive::`/`roots::`/
`leaves::` parameters answer set questions *about* the graph ("which chunks
depend on hash21") — the answer is still a flat set of chunk records.
`.tree` renders the graph structure — nesting is the payload, and its
[`tree_aligned`](../format/02_tree_aligned.md) output has no
column-projection or format-selection surface. Merging would put a command
with a 1-parameter surface and a structural output inside a group defined
by its shared 20-parameter set engine.

### Invariants

- Idempotent: identical input always produces identical output.
- No side effects outside stdout content and process exit code.
- Only `shader_chunks_core::CHUNKS` is consulted — no filesystem,
  environment, or network access.
- Child order and nesting mirror the declared `depends_on` metadata
  exactly — never alphabetized, never flattened.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.tree`](../command/04_tree.md) | Member — dependency tree or full forest |

**Membership:** 1 of the 8 commands; the partition across all groups is
stated in [`readme.md`](readme.md). A single-member group is deliberate —
the boundary is output-species (graph rendering), not command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/02_graph.md`](../../../tests/docs/cli/command_group/02_graph.md) | Group-level test specification |
| [`../../../../shader_chunks_query_core/tests/shader_chunks_query_core_test.rs`](../../../../shader_chunks_query_core/tests/shader_chunks_query_core_test.rs) | `tree_chunk_shows_fbm3_dependency_chain_in_order`, `tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |

### Typical Patterns

Inspect before composing: `tree <name>` to confirm what `compose` will
pull in transitively; bare `tree` to survey every entry-point chunk at
once.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
