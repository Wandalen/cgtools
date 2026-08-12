# Algorithm: Top-Level Statement Classification

### Scope

- **Purpose**: Determine, for each top-level statement in a compiled script, which of four roles it plays so the checker can allow or reject it by role and position.
- **Responsibility**: Document the classification decision tree, including how a call is detected under Rhai's own AST shapes (bare call, trailing implicit-return, dotted method call).
- **In Scope**: `stmt_kind()`, `call_expr()`, `role()`, and the final accept/reject pass in `check_top_level_is_declarative()`.
- **Out of Scope**: The property this classification exists to enforce (see [`invariant/001`](../invariant/001_top_level_bindings_convention.md), which states the WHAT; this document states the HOW).

### Abstract

A compiled Rhai script's top-level statement list must satisfy the declarative-bindings convention: only bindings, side-effect-free expressions, and a single trailing entry-point call are allowed. Enforcing that convention requires answering one question per statement — "what role does this statement play?" — before a single allow/reject decision can be made. The algorithm answers that question with a four-way classification (`Binding`, `PlainExpression`, `Call(name)`, `Imperative`) that is purely structural: it inspects the shape of the statement and its outermost expression, never what any called function actually does at runtime. The one nontrivial sub-problem is call detection: Rhai does not represent every call the same way at a statement's outermost node — a bare call, an implicit-return trailing call, and a dotted method call all need to resolve to the same "this statement is a call" answer, and operator arithmetic (which Rhai also implements as a function call under the hood) must specifically *not* count as one.

### Algorithm

For each top-level statement, in source order:

1. **Detect a call, if present** (`call_expr()`):
   - If the statement is itself a bare `Stmt::FnCall`, that is the call.
   - If the statement is `Stmt::Expr(inner)`, recurse into `inner`: an `Expr::FnCall` or `Expr::MethodCall` node is the call directly; an `Expr::Dot(binary, ..)` node recurses into `binary.rhs` (this is what lets a dotted method call — `t.update(0.5)`, or a chain of dots — resolve to its terminal call, since Rhai nests each dot's right-hand side one level behind the dot rather than exposing the call at the statement's own outermost node); any other expression shape yields no call.
   - Any other statement variant yields no call.
2. **Classify the role** (`role()`), combining the statement variant with step 1's result:
   - `Stmt::Var` (`let`/`const`) or `Stmt::Noop` → `Binding`, unconditionally — the initializer expression's own content is never inspected at this stage (see [`pitfall/002`](../pitfall/002_checker_is_structural_not_semantic.md) for why this is a known gap, not an oversight).
   - A call was found and `call.is_operator_call()` is true → `PlainExpression` — Rhai represents `+`/`-`/`*`/etc. as function calls internally, but arithmetic is declarative, not an imperative action, so it is deliberately classified alongside literals and variable references rather than as a `Call`.
   - A call was found and it is not an operator call → `Call(name)`, carrying the called function's name.
   - The statement is `Stmt::Expr` and no call was found → `PlainExpression` (a literal, a variable reference, or any other side-effect-free expression shape).
   - Anything else (loops, `if`, `while`, assignment, blocks, try/catch, break/return) → `Imperative`.
3. **Decide allow/reject by role and position**, walking the list once with the final index recorded up front:
   - `Binding` or `PlainExpression` → always allowed, at any position.
   - `Call("main")` → allowed only if this is the last statement in the list.
   - `Call(_)` for any other name, or `Imperative`, at any position, or `Call("main")` anywhere but last → rejected. The check returns immediately on the first rejection, carrying the offending statement's position (`rhai::Position`) and a human-readable kind label (`stmt_kind()` — a separate, simpler mapping from `Stmt` variant to a display string, used only for the error message, not for classification itself).

The algorithm never descends into a called function's body — `rhai::AST::statements()` itself only returns the top-level list, so imperative code nested inside `fn main() { .. }` is structurally invisible to this pass, which is precisely why it is permitted there.

### Invariants

| File | Relationship |
|------|--------------|
| [001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) | The property this classification's accept/reject decisions implement |

### Pitfalls

| File | Relationship |
|------|--------------|
| [002_checker_is_structural_not_semantic.md](../pitfall/002_checker_is_structural_not_semantic.md) | The two concrete gaps that follow from this algorithm inspecting shape only, never a call's actual effect |
| [005_optimization_level_simple_folds_trivial_top_level_constructs.md](../pitfall/005_optimization_level_simple_folds_trivial_top_level_constructs.md) | Rhai's optimizer can remove a statement before this algorithm ever classifies it |

### Sources

| File | Relationship |
|------|--------------|
| `src/top_level_lint.rs` | `stmt_kind()`, `Role`, `call_expr()`, `role()`, `check_top_level_is_declarative()` — the entire algorithm |

### Tests

| File | Relationship |
|------|--------------|
| `tests/example_convention_test.rs` | Exercises every role/position combination directly: bindings, operator expressions, bare/trailing/dotted calls, `main`-at-non-last-position, non-`main` calls, loops, `if` |
