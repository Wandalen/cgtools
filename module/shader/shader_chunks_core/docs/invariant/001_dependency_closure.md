# Invariant: Dependency Closure

A chunk set about to be composed contains everything its members need: no
`depends_on` entry names a chunk outside the set. Consumers can hold this at
build time, so a forgotten import fails `cargo check`, never the first
composed frame.

### Scope

- **Purpose**: Pin the completeness precondition composition stands on, and the compile-time check that moves its violation from runtime to build time.
- **Responsibility**: State the property, enumerate its compile-time and runtime enforcement, and record what breaks when it fails.
- **In Scope**: `dependency_closed` and the `const` assert form; the runtime `MissingDependency` backstop.
- **Out of Scope**: Cycle-freedom (reported by the same composition core as `CyclicDependency`, but a property of chunk authoring, not of set selection); how the set is then ordered (see [../algorithm/002_dependency_ordered_composition.md](../algorithm/002_dependency_ordered_composition.md)).

### Invariant Statement

For every chunk `c` in a set `S` and every name `d` in `c.depends_on`, there
exists a chunk in `S` whose `name` is `d`. Equivalently: `S` is transitively
complete — composition never has to reach outside it.

### Enforcement Mechanism

- **Compile time (the intended form)**: `dependency_closed( set )` is a
  `const fn` computing the property by scanning descriptor fields; a
  consumer holds the invariant with one line —
  `const _ : () = assert!( dependency_closed( MY_CHUNKS ) );` — which fails
  the build (rustc `E0080`) when the set is incomplete. The orrery consumer
  carries exactly this assert over its live set.
- **Runtime backstop**: for sets not checked in `const` position (untrusted
  CLI input), the composition core's `visit` reports the first unresolved
  name as `ComposeError::MissingDependency`, naming both the missing chunk
  and the chunk that required it; `set_compose` panics with that same
  message, `set_try_compose` returns it as `Err`.

### Violation Consequences

- Without the `const` assert, an incomplete set surfaces as a panic (or
  `Err`) only when composition first runs — in a browser consumer, that is
  app startup, not the developer's build.
- If composition's own check were also absent, the failure would degrade
  further: the concatenated WGSL simply lacks a function, and the pipeline
  fails validation at creation time with a WGSL-level "unknown identifier"
  — furthest from the edit that caused it.

### Example

Drop `chunk( "value_noise" )` from a set that keeps `chunk( "fbm3" )`
(whose manifest declares `depends_on: value_noise`) and the assert fails the
build:

```text
error[E0080]: evaluation panicked: assertion failed: dependency_closed(UNCLOSED)
  --> tests/compile_fail/unclosed_set.rs:12:16
   |
12 | const _ : () = assert!( dependency_closed( UNCLOSED ) );
```

That diagnostic is a committed snapshot — the `unclosed_set` trybuild
fixture builds exactly this program and asserts it fails exactly this way.
Verify live in the real consumer: delete the `chunk( "value_noise" )` line
in `examples/orrery/webgpu/src/shader_source.rs`, run
`cargo check -p orrery_webgpu`, watch its `dependency_closed` assert fail
with its custom message, revert.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/002_dependency_ordered_composition.md](../algorithm/002_dependency_ordered_composition.md) | The procedure this invariant is the success precondition for; its `visit` step is the runtime backstop |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/001_selective_const_import.md](../pattern/001_selective_const_import.md) | The import form that assembles the sets this invariant validates, in the same `const` failure channel |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `dependency_closed` (`const` check) and `visit`'s `MissingDependency` reporting |
| `examples/orrery/webgpu/src/shader_source.rs` (repo root) | The live `const` assert over `SCENE_CHUNKS` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/compile_fail/unclosed_set.rs` | Trybuild fixture pinning the incomplete-set build failure and its diagnostic |
| `tests/shader_chunks_core_test.rs` | `dependency_closed_is_false_when_a_dependency_is_missing_from_the_set`, `try_compose_set_reports_missing_dependency`, `compose_panics_on_missing_dependency` |
