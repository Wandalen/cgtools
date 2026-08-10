# Invariant: Top-Level Bindings Convention

A script's top level holds only declarative bindings and, as the final
statement, a single named entry-point call — every imperative construct
(loop, branch, mutation) must live inside a function.

### Scope

- **Purpose**: Pin that a compiled script's top-level statement list stays declarative-shaped — the property `top_level_lint` enforces and every example script must satisfy.
- **Responsibility**: State the property precisely, enumerate the real (not aspirational) enforcement mechanism, and record what breaks when it's violated.
- **In Scope**: The shape of `rhai::AST::statements()` — the top-level statement list only.
- **Out of Scope**: Semantic determinism of what runs inside `main()` (an authorial discipline, only spot-checkable — see [`docs/pattern/005_script_as_glue.md`](../../../../../docs/pattern/005_script_as_glue.md)'s "Determinism by discipline" cost); whether a `let` initializer or an expression's sub-terms hide a side-effecting call (see [`pitfall/002`](../pitfall/002_checker_is_structural_not_semantic.md)).

### Invariant Statement

Every statement in a compiled script's top-level list
(`rhai::AST::statements()`) is one of: a `let`/`const` binding; a
side-effect-free value-producing expression (a literal, a variable
reference, or an operator expression like `a + b`); or — only as the very
last statement, and only calling `main` by name — the single call that
kicks off execution. Any other top-level statement (a loop, `if`, `while`,
assignment, a call to anything other than `main`, or a call to `main` that
isn't last) is rejected.

### Enforcement Mechanism

- `check_top_level_is_declarative()` (`src/top_level_lint.rs`) walks
  `ast.statements()` and classifies each into one of four roles —
  `Binding`, `PlainExpression`, `Call( name )`, `Imperative` — rejecting on
  the first statement whose role isn't allowed at its position.
  `rhai::AST::statements()` returns only the top-level list; it never
  descends into `fn` bodies, which is precisely why imperative code nested
  inside `fn main() { ... }` is permitted — it only rejects imperative
  constructs sitting bare at top level, outside of any function.
- This relies on `rhai`'s `internals` feature (enabled in `Cargo.toml`) to
  expose `AST::statements()` and the `Stmt` enum; it costs no extra
  dependency (`internals = []` in `rhai`'s own manifest) and is the same
  surface `rhai`'s own `debugging` feature builds on.
- `call_expr()` recognizes a call whether it arrives as a bare
  `Stmt::FnCall`, a trailing (implicit-return) `Stmt::Expr( Expr::FnCall )`,
  or a dotted method call (`receiver.method( args )`) wrapped in one or
  more `Expr::Dot` layers — unwound recursively via the `rhs` link down to
  the terminal `Expr::MethodCall`/`Expr::FnCall`.
- Rhai operator calls (`+`, `-`, `*`, ...) classify as `PlainExpression`,
  not `Call`, even though Rhai represents arithmetic as a `FnCallExpr`
  under the hood (`FnCallExpr::is_operator_call()`) — arithmetic is
  declarative, not an imperative action.
- Every script under `examples/scene_script/*/src/*.rhai` is checked
  against this function in
  `tests/example_convention_test.rs::example_scripts_follow_declarative_top_level_convention`
  — a test-suite-visible regression gate, not just a hand-authored
  convention documented in prose.
- `build_engine()` (`src/engine.rs`) runs Rhai's default
  `OptimizationLevel::Simple`, which folds dead/no-op constructs (an
  empty-bodied, else-less `if`; an unused local `let`) before this checker
  ever inspects them — irrelevant to real scripts (which do real work
  inside their branches), but the reason this checker's own test suite must
  write non-trivial condition/body pairs to exercise the reject path for
  `if`.

### Violation Consequences

- A script that mutates state at top level makes the crate's own
  "declarative vs. imperative" distinction unreliable as a signal to a
  reader — the whole point of separating
  `examples/scene_script/f32x2_vector_arithmetic/`'s pattern (declarative,
  no `main()` at all) from
  `examples/scene_script/pingpong_animation/`'s pattern (imperative,
  confined to `main()`) collapses if either script can leak imperative code
  to the top level.
- Enforcement is structural and loud, not a silent runtime surprise: a
  violating script fails `check_top_level_is_declarative()` wherever the
  host calls it — for the tracked examples, that's a failing test.

### Patterns

| File | Relationship |
|------|--------------|
| [../../../../../docs/pattern/005_script_as_glue.md](../../../../../docs/pattern/005_script_as_glue.md) | The imperative script-form this invariant's convention applies to |

### Layers

| File | Relationship |
|------|--------------|
| [../../../../../docs/layer/006_l5_scene_script_and_runners.md](../../../../../docs/layer/006_l5_scene_script_and_runners.md) | The L5 layer contract `scene_script` realizes as script-as-glue |

### Sources

| File | Relationship |
|------|--------------|
| `src/top_level_lint.rs` | `check_top_level_is_declarative()`, `call_expr()`, `role()` — the enforcement itself |
| `src/engine.rs` | `build_engine()` — the `OptimizationLevel::Simple` engine every script is checked against |

### Tests

| File | Relationship |
|------|--------------|
| `tests/example_convention_test.rs` | `example_scripts_follow_declarative_top_level_convention` checks every real example script; the remaining 10 tests cover the checker's own accept/reject edge cases |
