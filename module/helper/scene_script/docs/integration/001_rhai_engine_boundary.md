# Integration: Rhai Engine Boundary

### Scope

- **Purpose**: Document how Rust host code and the embedded Rhai interpreter exchange types, calls, and errors at runtime.
- **Responsibility**: Describe the access method, error propagation, and version-compatibility posture of this boundary.
- **In Scope**: `Engine::compile`/`eval`, and the `register_type_with_name`/`register_fn`/`register_get` surface as the crossing point.
- **Out of Scope**: Why `rhai` was selected and its feature configuration (see [`dependency/001`](../dependency/001_rhai_internals_feature.md)).

### System Description

Rhai is an embeddable scripting language interpreter running fully in-process — no subprocess, no IPC, no network boundary. `scene_script` depends on the workspace-pinned `rhai` version with the `internals` feature enabled (see [`dependency/001`](../dependency/001_rhai_internals_feature.md)). It provides a dynamically-typed script runtime plus a Rust-native embedding API centered on `Engine` (the interpreter instance), `AST` (a compiled script), `Dynamic` (a type-erased script value), and `EvalAltResult` (the error type for anything that goes wrong during compilation or execution).

### Integration Points

- **Construction**: `engine_build()` ([`src/engine.rs`](../../src/engine.rs)) creates exactly one `Engine::new()` and registers all four bindings onto it — every script sharing that engine sees the same registered surface.
- **Registration** (the one-time, setup-side crossing): `Engine::register_type_with_name`, `register_fn`, and `register_get` are the sole channel through which Rust functionality becomes callable from a script — nothing is reachable from a script that was not explicitly registered this way (see [`pattern/001`](../pattern/001_manual_customtype_registration_for_foreign_types.md)).
- **Execution** (the per-script crossing): `Engine::compile(source) -> AST` produces a compiled, inspectable script — this is the `AST` [`algorithm/001`](../algorithm/001_top_level_statement_classification.md) walks. `Engine::eval::<T>(source) -> T` compiles, runs, and extracts a typed result in one call — the shape every test in `tests/engine_test.rs` uses.

### Error Handling

A type mismatch — a script constructs or returns a value that does not match the Rust type requested at the call site (e.g. evaluating as `F64x2` a script that actually produces an `F32x2`) — surfaces as `Box<EvalAltResult>` from `compile`/`eval`, carrying a human-readable message (confirmed: `f32x2_and_f64x2_are_distinct_types_not_interchangeable` asserts the message text contains `"type"`). No retry, fallback, or recovery logic exists anywhere in this crate for such errors — every call site across the test suite and `top_level_lint.rs` either propagates the `Result` or calls `.unwrap()`/`.expect()` directly. That is an appropriate posture for this crate's own tests and internal tooling, but it is not a runtime resilience policy a downstream consumer embedding `scene_script`'s engine should assume applies to their own use.

### Compatibility Requirements

This boundary is tied entirely to the workspace-pinned `rhai` version. The `internals`-feature surface this crate depends on (`AST::statements()`, `Stmt`) is not guaranteed stable across `rhai` versions by `rhai`'s own design intent (see [`dependency/001`](../dependency/001_rhai_internals_feature.md)'s Known Issues) — there is no version-detection, negotiation, or compatibility shim on `scene_script`'s side; an incompatible `rhai` upgrade would surface as a compile error against `top_level_lint.rs`, not a runtime failure.

### Algorithms

| File | Relationship |
|------|--------------|
| [001_top_level_statement_classification.md](../algorithm/001_top_level_statement_classification.md) | The consumer of the `AST` this boundary's compile step produces |

### APIs

| File | Relationship |
|------|--------------|
| [001_rhai_scripting_surface.md](../api/001_rhai_scripting_surface.md) | What becomes callable through the registration integration point |

### Dependencies

| File | Relationship |
|------|--------------|
| [001_rhai_internals_feature.md](../dependency/001_rhai_internals_feature.md) | Why `rhai` and its `internals` feature were selected, as distinct from this document's operational-boundary framing |

### Features

| File | Relationship |
|------|--------------|
| [001_rhai_scene_scripting.md](../feature/001_rhai_scene_scripting.md) | Navigational hub this runtime boundary supports |

### Sources

| File | Relationship |
|------|--------------|
| `src/engine.rs` | `engine_build()` — engine construction and registration |
| `src/top_level_lint.rs` | `check_top_level_is_declarative()` — the `AST` consumer |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engine_test.rs` | `f32x2_and_f64x2_are_distinct_types_not_interchangeable` — the one test directly exercising this boundary's error-handling behavior |
