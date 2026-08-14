# Pitfall: `OptimizationLevel::Simple` Folds Trivial Top-Level Constructs

### Scope

- **Purpose**: Warn that a top-level imperative construct can be optimized away before `check_top_level_is_declarative()` ever inspects it.
- **Responsibility**: Document the concrete folding behavior and how to write a test or script that isn't silently affected by it.
- **In Scope**: `OptimizationLevel::Simple`'s interaction with the checker, as it affects any new test written against `check_top_level_is_declarative()`.
- **Out of Scope**: The checker's own classification logic once it actually receives a statement (see [`algorithm/001`](../algorithm/001_top_level_statement_classification.md)).

### Trap

`engine_build()` constructs a plain `rhai::Engine::new()` ([`src/engine.rs`](../../src/engine.rs)), which defaults to `OptimizationLevel::Simple`. Optimization runs during `Engine::compile`, *before* `check_top_level_is_declarative()` ever inspects the resulting `AST` — so anything the optimizer can prove dead is already gone by the time the checker runs. Concretely: an empty-bodied, else-less `if` collapses regardless of its condition (taking the branch or not is observationally identical to not having it), and a body containing only an unused `let` is dead-code-eliminated down to empty first, collapsing the same way even when the source isn't textually empty.

### Failure

A test asserting the checker rejects something shaped like `if some_condition { let y = 1; }` can pass for the wrong reason, or fail to exercise the reject path at all — `Simple` folds the whole `if` away before the checker runs, leaving no `Stmt::If` to reject. This already happened once in this crate's own test suite: `checker_rejects_a_top_level_if` (`tests/example_convention_test.rs`) needed a condition the optimizer cannot evaluate (a script-defined function call — `Simple` never evaluates calls; only `OptimizationLevel::Full` does) and a body that mutates an outer binding read afterward, specifically so the construct survives optimization as a genuine `Stmt::If` for the checker to see.

### Mitigation

When writing a new test against `check_top_level_is_declarative()` for any construct the optimizer might fold, either: (a) give the condition a side effect the optimizer can't evaluate (a function call) and the body an externally-observable mutation, mirroring `checker_rejects_a_top_level_if`'s exact shape, or (b) compile with a non-default `OptimizationLevel` if what's actually under test is independent of folding behavior. Never assume a trivial-looking reject-case script reaches the checker exactly as written — verify by checking `violation.kind` matches the expected `Stmt` variant, the way every existing checker test already does.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_top_level_statement_classification.md](../algorithm/001_top_level_statement_classification.md) | The classification this pitfall's folding can prevent from ever running on a given statement |

### Sources

| File | Relationship |
|------|--------------|
| `src/engine.rs` | `engine_build()` — where `OptimizationLevel::Simple` is implicitly selected via `Engine::new()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/example_convention_test.rs` | `checker_rejects_a_top_level_if`'s doc comment records the exact folding behavior and why its condition/body are shaped the way they are |
