# Invariant Test :: Script-As-Data Purity

Source: [`../../../docs/invariant/004_script_as_data_purity.md`](../../../docs/invariant/004_script_as_data_purity.md)

A script bound by the script-as-data pattern contains no function or method
call anywhere in its AST — top-level or nested, not even a desugared
operator; enforced by `check_whole_ast_is_pure()` (`src/purity_lint.rs`).

### Test Cases

### IN-1: Valid pure data-form script is accepted

- **Given:** A script-as-data source whose entire AST — top-level and every
  nested position (`let` initializers, array elements, object-map values,
  control-flow bodies, script-defined function bodies) — contains no
  `FnCall`/`MethodCall` node, including desugared operators (e.g. orrery's
  real `scene.rhai`)
- **When:** The script is loaded via `scene_script`'s production
  compile-and-lint entry point for the data form
- **Then:** The invariant holds — the script compiles and evaluates
  successfully, identical output to today's direct compile path

### IN-2: A call expression anywhere in the AST is rejected before evaluation

- **Given:** A script-as-data source containing a `FnCall`/`MethodCall` node
  anywhere in its AST — top-level, nested inside a `let` initializer, or
  inside a script-defined function's body — whether a named call or a
  desugared operator (`+`, `==`, ...)
- **When:** The script is loaded via `scene_script`'s production
  compile-and-lint entry point for the data form
- **Then:** The invariant holds — the entry point returns `Err` identifying
  the impure call before any engine evaluation happens; the script never
  runs

### Cross-References

| File | Relationship |
|------|----------------|
| [`task/verified/416_scene_script_production_lint_enforcement.md`](../../../../../../task/verified/416_scene_script_production_lint_enforcement.md) | The task adding the production entry point this spec's cases target (T02, T04) |
| [`../../../docs/invariant/004_script_as_data_purity.md`](../../../docs/invariant/004_script_as_data_purity.md) | The invariant this spec covers |
