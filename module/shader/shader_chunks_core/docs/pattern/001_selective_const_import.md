# Pattern: Selective Const Import

An application names exactly the bundled chunks it uses — `chunk( "fbm3" )`
— and receives each one as compile-time data. Nothing unnamed enters the
set, and a name the registry doesn't know is a build failure at the call
site, never a runtime surprise.

### Scope

- **Purpose**: Give consumers deliberate, compile-time-checked selection instead of absorbing the whole bundled table or deferring name errors to runtime.
- **Responsibility**: Describe the import form, its failure mode, when it applies, and its trade against runtime lookup.
- **In Scope**: `chunk( name )` for importing from the bundled registry and `chunk_get_from( set, name )` for selecting out of any set, both in `const` position.
- **Out of Scope**: Where the registry comes from (see [../algorithm/001_registry_generation.md](../algorithm/001_registry_generation.md)); completeness of the assembled set (see [../invariant/001_dependency_closure.md](../invariant/001_dependency_closure.md)).

### Problem

A shader uses a handful of chunks, not the whole collection. Consuming
`CHUNKS` wholesale couples every shader to the collection's growth — a chunk
added later silently enters every consumer's composed source. And resolving
names at runtime (`chunk_get`) means a typo'd name surfaces as a `None` or
panic on the first composed frame, in the browser, instead of on the
developer's machine.

### Solution

Selection happens in `const` position. `chunk( name )` is a `const fn` that
scans the bundled table during compile-time evaluation and returns the
matching descriptor **by value**; an unknown name takes the `panic!` branch,
which in `const` evaluation is a hard build error (rustc `E0080`) pointing
at the exact call site. `chunk_get_from( set, name )` is the same `const`
lookup generalized to any caller-supplied set — imported, local, or mixed —
so selecting a row back out of an assembled set is equally compile-time.
A `const` byte-wise `str_eq` underpins both, because `==` on `&str` is not
callable in const context.

### Applicability

Use whenever the consumer knows its chunk selection at compile time — the
normal case for an application's shaders. Do not use for dynamic,
user-driven selection (a CLI, an editor): there the untrusted-input twins
are the right tool — runtime `chunk_get` plus `try_compose_set`, which
report bad names as values instead of failing a build.

### Consequences

- A typo'd import cannot survive to runtime: the build fails naming the
  line. Pinned as a real failing build by the `unknown_chunk_name` trybuild
  fixture.
- Selection is explicit and closed — the composed shader's contents change
  only when its own source names change.
- The descriptor is `Copy` and returned by value: the imported `const` is
  plain data, usable in the consumer's own `const` tables.
- The guarantee is positional: only `const` position turns the panic into a
  build error. Calling `chunk` with a bad name at runtime panics at runtime
  (the `chunk_panics_for_unknown_name_at_runtime` test pins that too).

### Example

```rust
use shader_chunks_core::{ ChunkDescriptor, chunk };

const MY_CHUNKS : &[ ChunkDescriptor ] =
&[
  chunk( "hash21" ),
  chunk( "value_noise" ),
];
```

Misspell a name — `chunk( "hash12" )` — and `cargo check` fails with:

```text
error[E0080]: evaluation panicked: unknown chunk name — see CHUNKS for the bundled set
 --> src/…
  | const BROKEN : … = shader_chunks_core::chunk( "hash12" );
  |                    ^^^ evaluation of `BROKEN` failed inside this call
```

Verify in one minute: change any `chunk( … )` string in
`examples/orrery/webgpu/src/shader_source.rs`, run
`cargo check -p orrery_webgpu`, watch the exact line fail, revert.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_registry_generation.md](../algorithm/001_registry_generation.md) | Generates the bundled table this pattern imports from |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_dependency_closure.md](../invariant/001_dependency_closure.md) | Validates, also at compile time, that the selection this pattern assembles is complete |

### Patterns

| File | Relationship |
|------|--------------|
| [002_crate_local_chunk.md](002_crate_local_chunk.md) | The complementary row source: locally-defined chunks mixing into the same set |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `chunk`, `chunk_get_from`, and the `const` `str_eq` they stand on |
| `examples/orrery/webgpu/src/shader_source.rs` (repo root) | Known use: `SCENE_CHUNKS` imports four bundled chunks by name |

### Tests

| File | Relationship |
|------|--------------|
| `tests/compile_fail/unknown_chunk_name.rs` | Trybuild fixture pinning the typo'd-import build failure and its diagnostic |
| `tests/shader_chunks_core_test.rs` | `chunk_imports_a_bundled_descriptor_by_value_in_const_position`, `chunk_get_from_resolves_imported_and_local_rows_of_a_mixed_set`, `chunk_panics_for_unknown_name_at_runtime` |
