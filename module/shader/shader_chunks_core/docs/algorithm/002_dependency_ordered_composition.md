# Algorithm: Dependency-Ordered Composition

One sort core turns any chunk set into a single WGSL source, concatenated
dependency-before-dependent regardless of the order the chunks were passed
in. Every composition entry point — `compose`, `try_compose`, `set_compose`,
`set_try_compose` — is a front door onto this same procedure.

### Scope

- **Purpose**: Guarantee that a caller never has to know or maintain the correct concatenation order of the chunks it selects.
- **Responsibility**: Describe the normalization step, the depth-first sort, its ordering and failure properties, and a worked example.
- **In Scope**: The `entries_sort_and_join`/`visit` core, both input paths onto it (raw text and descriptors), and the panic-vs-`Result` twin contract.
- **Out of Scope**: The set-completeness precondition and its compile-time check (see [../invariant/001_dependency_closure.md](../invariant/001_dependency_closure.md)); where the composed sets come from (see [../pattern/002_crate_local_chunk.md](../pattern/002_crate_local_chunk.md)).

### Abstract

A chunk declares what it needs by name in its `//@ depends_on:` manifest
line, mirrored into the `depends_on` field of its descriptor. Composition
reads only those declarations — never the WGSL bodies — to order the set so
that every function is declared before the first chunk that calls it, which
is what WGSL requires of the concatenated result. The procedure is
deterministic: same set, same output, every run.

Two input paths normalize into the same core: `compose`/`try_compose` take
raw WGSL texts and parse each one's manifest; `set_compose`/
`set_try_compose` take `ChunkDescriptor`s and read the fields directly, with
no manifest parsing at runtime. Each path has a panicking form (for sets the
caller trusts — a failure is an authoring bug) and a `Result` form returning
`ComposeError` (for untrusted sets, e.g. a CLI taking user input).

### Algorithm

1. **Normalize**: reduce each input chunk to `( name, depends_on, wgsl )` —
   parsed from the text on the `compose` path, copied from descriptor fields
   on the `set_compose` path.
2. **Visit in given order**: for each entry, run a depth-first `visit` of
   its name.
3. **`visit( name )`**: if the chunk is already emitted, return. If it is on
   the visiting stack, fail with `CyclicDependency`, carrying the stack
   trail that closed the cycle. If no entry in the set has this name, fail
   with `MissingDependency`, naming both the missing chunk and the chunk
   that required it. Otherwise push the name onto the visiting stack, visit
   each of its `depends_on` entries in declared order, pop, and emit the
   chunk.
4. **Join**: concatenate the emitted WGSL texts, in emission order,
   separated by blank lines.

Post-order emission is what yields dependency-before-dependent; chunks with
no ordering relation between them keep their first-visit (input) order, so
the output is stable, not merely valid.

### Example

The orrery scene passes its set as `hash21`, `value_noise`, `fbm3`,
`fullscreen_triangle`, `scene_fragment` (the local fragment depends on
`hash21`, `fbm3`, `fullscreen_triangle`; `fbm3 → value_noise → hash21`).
Emission order: `hash21`, `value_noise`, `fbm3`, `fullscreen_triangle`,
`scene_fragment`. Reverse the input set completely and the visit of
`scene_fragment` pulls its dependencies in first — the output order is
unchanged. Verify both claims by running the pinned tests:
`compose_orders_dependencies_before_dependents_regardless_of_input_order`
in this crate and `assembled_shader_orders_dependencies_before_dependents`
in the orrery consumer, e.g.
`cargo nextest run -p shader_chunks_core -p orrery_webgpu orders_dependencies`.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_dependency_closure.md](../invariant/001_dependency_closure.md) | This algorithm's success precondition; `MissingDependency` is its runtime violation report |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_crate_local_chunk.md](../pattern/002_crate_local_chunk.md) | Local rows compose seamlessly because this algorithm reads only descriptor fields, never provenance |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `compose`/`try_compose`, `set_compose`/`set_try_compose`, and the shared `entries_sort_and_join`/`visit` core |
| `examples/orrery/webgpu/src/shader_source.rs` (repo root) | Live call site: `assemble()` composes the mixed scene set |

### Tests

| File | Relationship |
|------|--------------|
| `tests/shader_chunks_core_test.rs` | Ordering, cycle, and missing-dependency contracts for both input paths (`compose_orders_…`, `compose_set_orders_…`, `try_compose_…` cases) |
| `examples/orrery/webgpu/tests/shader_source_test.rs` (repo root) | `assembled_shader_orders_dependencies_before_dependents` plus `assembled_wgsl_parses_and_validates` (naga) prove composed output is ordered and valid WGSL |
