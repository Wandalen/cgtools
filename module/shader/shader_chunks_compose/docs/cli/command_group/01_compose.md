# Command Group :: 3. Compose

### Pattern

WGSL text production: resolve the named chunks plus their declared
dependencies into dependency order and print the concatenated WGSL source
— an artifact preview, not a metadata view.

### Purpose

Let a shader author preview exactly the WGSL text that
`shader_chunks_core::try_compose` would hand to a real render pipeline,
before wiring it in — or, with `out::<path>`, capture that same text as
a committable `.wgsl` file.

### Semantic Coherence Test

"The member command produces WGSL source text from the compiled-in chunk
registry." `.compose` is the only command whose output is shader code
rather than information *about* shader code.

### Why NOT Merge Into Query

`.compose`'s failure modes are graph-semantic, not filter-semantic: a
missing dependency or a cyclic dependency fails composition even though
every named chunk individually exists. Its output
([`plain_text`](../format/01_plain_text.md), raw WGSL) has no fields to
project, no rows to sort, no format to select. It shares the registry with
[Query](../../../../shader_chunks_query/docs/cli/command_group/01_query.md) but nothing of the query surface.

### Invariants

- Idempotent: identical input always produces identical output.
- No side effects beyond process exit code and, depending on `out::`,
  either stdout content or a single file write — the composed WGSL is
  printed by default, or written to `out::<path>` when given, never
  both at once.
- Output order is dependency order, regardless of input order.
- Missing dependencies and cycles fail loudly with a non-zero exit —
  never partial output.
- With `out::` given, the write is only ever attempted after
  composition already succeeded — a missing dependency, cyclic
  dependency, or unknown chunk (exit 1) never reaches the write step,
  so a failed `compose` never leaves a partial or stale file at
  `out::`. A write-side failure (e.g. a missing parent directory) is a
  distinct `Io` error, exit 2 — the only case this command exits 2.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.compose`](../command/01_compose.md) | Member — dependency-ordered WGSL preview |

**Membership:** 1 of the 9 commands across the `shader_chunks` family; the
full partition across all 7 command groups (spanning all 6 leaf CLIs) is
stated in [the family index](../../../../shader_chunks/docs/cli/readme.md).
A single-member group is deliberate — the boundary is output-species (WGSL
text production), not command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/01_compose.md`](../../../tests/docs/cli/command_group/01_compose.md) | Group-level test specification |
| [`../../../../shader_chunks_compose/tests/shader_chunks_compose_test.rs`](../../../../shader_chunks_compose/tests/shader_chunks_compose_test.rs) | `compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`, `compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`, `try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture`, `compose_write_writes_the_composed_text_and_returns_a_byte_count_summary`, `compose_write_to_an_unwritable_path_is_an_io_error_with_exit_code_2`, `subprocess_compose_writes_the_file_and_prints_the_summary`, `subprocess_compose_without_out_prints_composed_text_to_stdout` |

### Typical Patterns

Discover with [Query](../../../../shader_chunks_query/docs/cli/command_group/01_query.md), confirm structure with
[Graph](../../../../shader_chunks_query/docs/cli/command_group/02_graph.md), then `compose <names...>` to preview the final WGSL.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
