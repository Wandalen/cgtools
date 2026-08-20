# Pitfall: Checker Enforcement Is Structural, Not Semantic

### Scope

- **Purpose**: Warn a script author or reviewer that `check_top_level_is_declarative()` passing is not proof a top-level statement is actually side-effect-free — it only proves the statement has an allowed *shape*.
- **Responsibility**: Document the two concrete, verified ways a side-effecting call currently passes the checker undetected, and the practical mitigation.
- **In Scope**: What currently passes `check_top_level_is_declarative()` despite calling a (potentially side-effecting) host or script function at top level.
- **Out of Scope**: The checker's actual, correctly-enforced shape rules (see [`invariant/001`](../invariant/001_top_level_bindings_convention.md), which this doc's Trap section only describes the edge of).

### Trap

It is natural to read a passing `check_top_level_is_declarative()` as "this
script's top level has no side effects" — but the checker only inspects
each top-level statement's own *shape* (which `rhai::Stmt`/`Expr` variant
it is), never what a call inside that shape actually *does* at runtime.
Two concrete, verified gaps follow directly from that:

1. **A `let` initializer can call anything.** Every `Stmt::Var` (`let`/
   `const`) is unconditionally classified `Role::Binding` and allowed —
   the checker never looks at the initializer expression's own content.
   `let x = some_host_fn_with_side_effects();` passes exactly like
   `let x = 1;` does.
2. **A call nested inside a larger expression is invisible.** `call_expr()`
   only inspects a statement's own outermost expression node. If that node
   is an operator call (`FnCallExpr::is_operator_call()` — Rhai represents
   `+`, `-`, `*`, etc. as function calls under the hood) or any other
   non-call expression shape, whatever calls are nested inside its
   *arguments* are never separately walked. `not_main() + 1` as a trailing
   top-level statement classifies as `Role::PlainExpression` (the outer
   `+` is an operator call) and is allowed — `not_main()`, the left
   operand, is never itself classified.

Both were confirmed directly against the checker (not assumed): compiling
`"fn not_main() { 1 } not_main() + 1"` and
`"fn not_main2() { 1 } let x = not_main2(); x"` and calling
`check_top_level_is_declarative()` on each returns `Ok(())` in both cases.

### Failure

A script author relying on "the checker passed" as a proxy for "top level
is safe/declarative" can ship a script whose top level quietly calls a
mutating host binding — through a `let` initializer, or buried in an
arithmetic expression — with no error, no warning, and no test failure
tied to this crate's own convention checking. The failure mode is the
same class described in
[`invariant/001`](../invariant/001_top_level_bindings_convention.md)'s
Violation Consequences (the declarative/imperative distinction becomes
unreliable as a signal), except here it happens *silently*, without even
the loud, structural rejection the checker provides for a bare top-level
loop or `if`.

### Mitigation

Treat a passing checker as proof of *shape* only, never of *effect* — the
same posture `tilemap_scene`'s
[pitfall/001](../../../tilemap_scene/docs/pitfall/001_load_time_validation_partially_enforced.md)
recommends for its own partially-enforced validation. Concretely: keep
`let` initializers in example/reference scripts to genuinely pure,
host-registered constructors (`f32x2(...)`, `tween(...)`) — this crate
does register one mutating binding (`.update(...)` on a tween, which
advances its internal time state), but no tracked example currently calls
it from a `let` initializer or buries it in a larger expression, so the
gap is theoretical for the tracked examples today, not yet exploited by
any of them — and treat any future host binding used that way (or any
binding that performs a real side effect: I/O, logging, mutating shared
state) as something to document and code-review by hand at the call site,
not something this checker will ever catch.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_top_level_statement_classification.md](../algorithm/001_top_level_statement_classification.md) | The classification logic whose structural, not semantic, enforcement this pitfall documents the edge of |

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_rhai_scene_scripting.md](../feature/001_rhai_scene_scripting.md) | Navigational hub this pitfall's warning serves |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) | The convention this checker enforces; this pitfall documents the edge of that enforcement |

### Sources

| File | Relationship |
|------|--------------|
| `src/top_level_lint.rs` | `role()`'s unconditional `Role::Binding` for `Stmt::Var`; `call_expr()`'s single-outermost-node inspection |

### Tests

No dedicated regression test pins either gap as passing — a test asserting
the current (silent-accept) behavior would itself be documenting the gap
rather than the intended contract, so none was added; both were instead
verified once, ad hoc, while writing this doc (see Trap above for the
exact scripts and result).
