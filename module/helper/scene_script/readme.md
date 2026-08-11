# scene_script

Rhai scripting glue for describing 2D scenes and driving their animation.

The same script language serves both roles: a *data format* for a scene
(building values from vector arithmetic and returning them) and *executable
glue* that drives the host imperatively (loops, branches, callbacks). See
[`examples/scene_script/`](../../../examples/scene_script/) at the workspace
root for one worked script of each kind. The top-level bindings convention
and known pitfalls are documented in
[`docs/`](docs/definition/readme.md); this crate is the reference
implementation.

## Responsibility Table

| File | Responsibility |
|------|-----------------|
| `src/lib.rs` | Crate entry point; re-exports each layer plus `rhai` itself. |
| `src/vector_binding.rs` | Registers `ndarray_cg::F32x2` and `F64x2` into a `rhai::Engine`: constructors, `.x`/`.y`, `+`/`-`/`*` operators. |
| `src/tween_binding.rs` | Registers `animation::Tween< F32x2 >` and `Tween< F64x2 >` into a `rhai::Engine`: `tween(...)` constructor, `.update`/`.value`/`.is_completed`. |
| `src/engine.rs` | `build_engine()` — a `rhai::Engine` with all four bindings pre-registered. |
| `src/top_level_lint.rs` | `check_top_level_is_declarative()` — rejects imperative code sitting outside of a function. |
| `tests/engine_test.rs` | Smoke tests for all four registrations, plus a distinctness check between `F32x2` and `F64x2`. |
| `tests/example_convention_test.rs` | Checks every `examples/scene_script/*/src/*.rhai` script against `check_top_level_is_declarative()`, plus the checker's own accept/reject cases. |
| `docs/` | Design documentation as typed doc definitions — see [docs/definition/readme.md](docs/definition/readme.md) |

Worked examples live at the workspace root, not under this crate — see
`examples/scene_script/f32x2_vector_arithmetic/` (declarative pattern: a
script builds a value from `F32x2` arithmetic and returns it) and
`examples/scene_script/pingpong_animation/` (imperative pattern: a script
simulates ball/paddle motion with loops, branches and vector arithmetic,
calling back into the host per tick; the host then drives a real
`animation::Tween` between two recorded frames).

## Top-level bindings convention

A script's top level may only contain `let`/`const` bindings, a bare
value-producing expression, and — as the final statement only — a single
call that kicks off execution (e.g. `main( ... )`); all imperative code
(loops, branches, mutation) must live inside a function.
`check_top_level_is_declarative()` enforces this structurally, not
semantically — see
[`docs/invariant/001_top_level_bindings_convention.md`](docs/invariant/001_top_level_bindings_convention.md)
for the precise statement and enforcement mechanism, and
[`docs/pitfall/`](docs/pitfall/readme.md) for the convention's known gaps
and Rhai's outer-scope-visibility surprise.

## Naming convention

Rhai-facing type and constructor names always mirror the real Rust
identifier (`F32x2` / `f32x2(...)`, `F64x2` / `f64x2(...)`), never a
generic alias like `Vec2` — see `ndarray_cg::vector` for the full
`{Element}x{Arity}` family this extends to if more element types or
arities are registered later. `F32x2` and `F64x2` are registered side by
side under distinct names so a script can pick whichever float precision
it needs; Rhai's own `FLOAT` is `f64`-only (see § Why bindings are
registered manually), so `F64x2` needs no boundary cast while `F32x2`
casts at the edge.

## Why bindings are registered manually

`F32x2` and `animation::Tween` are both foreign to this crate, so
implementing Rhai's `CustomType` trait on them directly would violate
Rust's orphan rule. Both are registered by hand via
`Engine::register_type_with_name` + `Engine::register_fn` instead.
