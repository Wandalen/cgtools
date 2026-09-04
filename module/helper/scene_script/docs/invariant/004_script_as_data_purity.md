# Invariant: Script-As-Data Purity

A script bound by the script-as-data pattern contains no function or method
call anywhere in its AST — top-level or nested, not even a desugared
operator; the document's values are the only content, and everything that
executes lives outside the script, in the compiler that reads it.

### Scope

- **Purpose**: Pin the whole-AST no-call guarantee [`docs/pattern/004_script_as_data.md`](../../../../../docs/pattern/004_script_as_data.md) requires but `top_level_lint`'s structural, top-level-only check cannot verify — the property `check_whole_ast_is_pure()` enforces.
- **Responsibility**: State the property precisely, enumerate the real enforcement mechanism, and record what breaks when it's violated.
- **In Scope**: The shape of the entire compiled script AST — every statement and expression, top-level and nested inside `let` initializers, array elements, object-map values, control-flow bodies, and script-defined function bodies.
- **Out of Scope**: The top-level-only shape rules `check_top_level_is_declarative()` enforces for the separate script-as-glue convention (see [`invariant/001`](001_top_level_bindings_convention.md)); non-call side channels such as a `const` referencing an engine-registered constant — structural, not semantic, the same class of gap [`pitfall/002`](../pitfall/002_checker_is_structural_not_semantic.md) documents for the top-level checker.

### Invariant Statement

No statement or expression anywhere in a compiled script's AST — top-level,
nested inside a `let` initializer, an array element, an object-map value,
the body of a block/`if`/loop/`switch`/`try`-`catch`, or a script-defined
function's body — is a `FnCall` or `MethodCall`, whether the call is to a
named function or a desugared operator (`+`, `==`, ...). A script satisfying
this invariant is pure data: a serialized value the compiler reads, never a
program the engine runs.

### Enforcement Mechanism

- `check_whole_ast_is_pure()` (`src/purity_lint.rs`) delegates traversal to
  `rhai::AST::walk`, which recursively visits every `Stmt`/`Expr` node
  reachable from both `ast.statements()` (the top-level list) and every
  script-defined function's body (`ast.iter_fn_def()`) — the second source
  is exactly what `ast.statements()` alone never reaches, and why
  `check_top_level_is_declarative()` cannot itself prove this stronger
  property.
- This relies on the same `rhai` `internals` feature `top_level_lint.rs`
  already depends on (enabled in `Cargo.toml`) to expose `AST::walk`,
  `ASTNode`, and the `Stmt`/`Expr` enums.
- The walk callback matches only `Stmt::FnCall`/`Expr::FnCall`/
  `Expr::MethodCall`; it stops and returns the first one found, by
  `Position` and function/operator name. No other node kind is inspected,
  so no exception exists for operator-desugared calls
  (`FnCallExpr::is_operator_call()` is never consulted) — unlike
  `check_top_level_is_declarative()`'s `role()`, which deliberately
  reclassifies operator calls as `PlainExpression`.
- `rhai::AST::walk` visits a node's own position in the tree before
  recursing into its children (pre-order) — the outermost call in any
  expression is what gets reported, even when it wraps further calls in its
  own arguments (e.g. `f(g())` reports `f`, not `g`).
- Traversal order matters for authoring correct fixtures: top-level
  statements are visited before any function body's, so a top-level
  entry-point call sitting before an unrelated nested violation would be
  reported instead of it — confirmed empirically while writing
  `tests/purity_lint_test.rs::rejects_a_call_two_blocks_deep_inside_a_function_body`,
  whose own doc comment records the concrete case this forced.

### Violation Consequences

- A script that fails this invariant is not the kind of document
  [`docs/pattern/004_script_as_data.md`](../../../../../docs/pattern/004_script_as_data.md)'s
  determinism guarantee covers — "same script → same frames" stops being a
  construction guarantee for it, since it is a program, not a fact.
- Enforcement is structural and loud, but test-suite-scoped, not a
  production gate: no loader calls `check_whole_ast_is_pure()` at script-load
  time (`examples/orrery/webgpu/src/scene.rs` compiles and evaluates
  `scene.rhai` directly, with no purity check in that path). The property is
  instead proven against the real, shipping `scene.rhai` content by
  `purity_lint_test.rs::accepts_the_real_orrery_scene_script_end_to_end`
  (`include_str!` of the same file `scene.rs` bundles), so a violation
  surfaces the next time the test suite runs, not at actual script-load
  time.

### Features

| File | Relationship |
|------|--------------|
| [001_rhai_scene_scripting.md](../feature/001_rhai_scene_scripting.md) | Navigational hub this invariant constrains |

### Layers

| File | Relationship |
|------|--------------|
| [../../../../../docs/layer/006_l5_scene_script_and_runners.md](../../../../../docs/layer/006_l5_scene_script_and_runners.md) | The L5 layer contract `scene_script` realizes as script-as-data |

### Patterns

| File | Relationship |
|------|--------------|
| [../../../../../docs/pattern/004_script_as_data.md](../../../../../docs/pattern/004_script_as_data.md) | The pattern this invariant gives a checkable, enforced form to |

### Sources

| File | Relationship |
|------|--------------|
| `src/purity_lint.rs` | `check_whole_ast_is_pure()`, `call_in_node()` — the enforcement itself |

### Tests

| File | Relationship |
|------|--------------|
| `tests/purity_lint_test.rs` | One accept case (a pure literal document) plus four reject cases spanning operator, named, and method calls, and recursion into a script-defined function's body |
