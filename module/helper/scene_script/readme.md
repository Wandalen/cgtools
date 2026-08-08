# scene_script

Rhai scripting glue for describing 2D scenes and driving their animation.

The same script language serves both roles: a *data format* for a scene
(building values from vector arithmetic and returning them) and *executable
glue* that drives the host imperatively (loops, branches, callbacks). See
`examples/` for one worked script of each kind.

## Responsibility Table

| File | Responsibility |
|------|-----------------|
| `src/lib.rs` | Crate entry point; re-exports each layer plus `rhai` itself. |
| `src/vector_binding.rs` | Registers `ndarray_cg::F32x2` into a `rhai::Engine`: constructor, `.x`/`.y`, `+`/`-`/`*` operators. |
| `src/tween_binding.rs` | Registers `animation::Tween< F32x2 >` into a `rhai::Engine`: `tween(...)` constructor, `.update`/`.value`/`.is_completed`. |
| `src/engine.rs` | `build_engine()` — a `rhai::Engine` with both bindings pre-registered. |
| `examples/f32x2_vector_arithmetic.rs` + `.rhai` | Declarative pattern: a script builds a value from `F32x2` arithmetic and returns it. |
| `examples/pingpong_animation.rs` + `.rhai` | Imperative pattern: a script simulates ball/paddle motion with loops, branches and vector arithmetic, calling back into the host per tick; the host then drives a real `animation::Tween` between two recorded frames. |
| `tests/engine_test.rs` | Smoke tests for both registrations. |

## Naming convention

Rhai-facing type and constructor names always mirror the real Rust
identifier (`F32x2` / `f32x2(...)`), never a generic alias like `Vec2` —
see `ndarray_cg::vector` for the full `{Element}x{Arity}` family this
extends to if more element types or arities are registered later.

## Why bindings are registered manually

`F32x2` and `animation::Tween` are both foreign to this crate, so
implementing Rhai's `CustomType` trait on them directly would violate
Rust's orphan rule. Both are registered by hand via
`Engine::register_type_with_name` + `Engine::register_fn` instead.
