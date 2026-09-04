# Invariant Test :: Top-Level Bindings Convention

Source: [`../../../docs/invariant/001_top_level_bindings_convention.md`](../../../docs/invariant/001_top_level_bindings_convention.md)

A compiled script's top-level statement list stays declarative-shaped: only
`let`/`const` bindings, side-effect-free expressions, and a single trailing
call to `main` are permitted; enforced by `check_top_level_is_declarative()`
(`src/top_level_lint.rs`).

### Test Cases

### IN-1: Valid glue-form script is accepted

- **Given:** A script-as-glue source whose top-level statement list contains
  only `let`/`const` bindings, side-effect-free expressions, and a single
  trailing call to `main` (e.g. `pingpong_animation.rhai`,
  `f32x2_vector_arithmetic.rhai`)
- **When:** The script is loaded via `scene_script`'s production
  compile-and-lint entry point for the glue form
- **Then:** The invariant holds — the script compiles and evaluates
  successfully, identical to today's direct `engine_build()` + compile path

### IN-2: Imperative top-level statement is rejected before evaluation

- **Given:** A script-as-glue source containing an imperative construct (a
  loop, `if`, assignment, or a call to anything other than `main`) sitting
  bare at the top level, outside any function
- **When:** The script is loaded via `scene_script`'s production
  compile-and-lint entry point for the glue form
- **Then:** The invariant holds — the entry point returns `Err` identifying
  the lint violation before any engine evaluation happens; the script never
  runs

### Cross-References

| File | Relationship |
|------|----------------|
| [`task/verified/416_scene_script_production_lint_enforcement.md`](../../../../../../task/verified/416_scene_script_production_lint_enforcement.md) | The task adding the production entry point this spec's cases target (T01, T03, T05) |
| [`../../../docs/invariant/001_top_level_bindings_convention.md`](../../../docs/invariant/001_top_level_bindings_convention.md) | The invariant this spec covers |
