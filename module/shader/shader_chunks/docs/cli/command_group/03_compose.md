# Command Group :: 3. Compose

### Pattern

WGSL text production: resolve the named chunks plus their declared
dependencies into dependency order and print the concatenated WGSL source
— an artifact preview, not a metadata view.

### Purpose

Let a shader author preview exactly the WGSL text that
`shader_chunks_core::try_compose` would hand to a real render pipeline,
before wiring it in.

### Semantic Coherence Test

"The member command produces WGSL source text from the compiled-in chunk
registry." `.compose` is the only command whose output is shader code
rather than information *about* shader code.

### Why NOT Merge Into Query

`.compose`'s failure modes are graph-semantic, not filter-semantic: a
missing dependency or a cyclic dependency fails composition even though
every named chunk individually exists. Its output
([`plain_text`](../format/03_plain_text.md), raw WGSL) has no fields to
project, no rows to sort, no format to select. It shares the registry with
[Query](01_query.md) but nothing of the query surface.

### Invariants

- Idempotent: identical input always produces identical output.
- No side effects outside stdout content and process exit code — the
  composed WGSL is printed, never written to a file.
- Output order is dependency order, regardless of input order.
- Missing dependencies and cycles fail loudly with a non-zero exit —
  never partial output.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.compose`](../command/05_compose.md) | Member — dependency-ordered WGSL preview |

**Membership:** 1 of the 6 commands; the partition across all groups is
stated in [`readme.md`](readme.md). A single-member group is deliberate —
the boundary is output-species (WGSL text production), not command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/03_compose.md`](../../../tests/docs/cli/command_group/03_compose.md) | Group-level test specification |
| [`../../../tests/shader_chunks_test.rs`](../../../tests/shader_chunks_test.rs) | `compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`, `compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`, `try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |

### Typical Patterns

Discover with [Query](01_query.md), confirm structure with
[Graph](02_graph.md), then `compose <names...>` to preview the final WGSL.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
