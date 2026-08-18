# BUG-351: `top_level_lint`'s `call_in_expr()` checks only a dot chain's `.rhs`, silently missing a call in the chain's `.lhs` receiver when the tail is a plain property read

- **Severity:** Medium (no crash, no attacker-controlled input, and no currently-tracked example
  script exercises the affected shape -- but a real, verified logic defect in the one function
  whose entire purpose is enforcing this crate's declarative-top-level safety net; a violating
  script it should reject instead passes silently, exactly the "silent, not loud" failure mode
  `docs/invariant/001`'s own Violation Consequences section says the enforcement mechanism is
  designed to avoid)
- **state:** Verified
- **Affects:** Every call to `check_top_level_is_declarative()` on a script whose sole or
  trailing top-level statement is a Rhai dot/index chain carrying a real, non-`main`,
  non-operator call in the chain's receiver (`.lhs`, at any nesting depth) with a non-call tail
  (`.rhs`) -- e.g. `trigger().x`. No currently-tracked example script
  (`examples/scene_script/*/src/*.rhai`) exercises this shape today (confirmed by direct read),
  so no committed content is currently affected -- the gap was latent until this session's fix.
- **Component:** `module/helper/scene_script` (`src/top_level_lint.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **Fix Task:** [386](../../verifying/386_register_scene_script_top_level_lint_dot_chain_lhs_call_detection_fix_closes_bug351.md)
- **Related Bugs:** None -- independent of BUG-230 (`with_repeat`'s `i64`->`i32` cast, same
  crate, unrelated function/root cause). Independent of the two still-open gaps documented in
  `docs/pitfall/002_checker_is_structural_not_semantic.md` (a `let` initializer's own content is
  never inspected; a call nested inside a larger expression's *arguments* is invisible) -- this
  bug is a 3rd, previously-undocumented gap in the same checker, and this fix closes only this
  one gap. See Impact below.

## Symptom

```rust
// pre-fix -- `top_level_lint.rs`'s `call_in_expr()`, the `Expr::Dot` arm (was line 100)
rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ),
```

```
$ cd module/helper/scene_script && cargo test -p scene_script --test example_convention_test -- --nocapture
...
thread 'checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read' (2960736) panicked at module/helper/scene_script/tests/example_convention_test.rs:226:6:
a call sitting in a dot chain's receiver must be rejected even when the chain's own tail is a plain property read: ()
```

`check_top_level_is_declarative()` on a script whose sole top-level statement is `trigger().x` --
`trigger()` is a real, non-`main`, non-operator function call sitting in the dot chain's
RECEIVER (`.lhs`); `.x` (the chain's own tail, `.rhs`) is a plain property read, not a call.
Pre-fix, this returns `Ok( () )` -- the trailing `: ()` in the panic line above is
`.expect_err()`'s own Debug-formatted payload, proving the call returned `Ok` rather than the
expected `Err`. The statement is silently misclassified `Role::PlainExpression` (allowed at any
top-level position) instead of `Role::Call( "trigger" )` (rejected, since `trigger` isn't
`main`).

## Impact

**Who is affected:** Any caller of `scene_script::check_top_level_is_declarative()` on a script
whose sole or trailing top-level statement is a Rhai dot/index chain where a real, non-operator,
non-`main` call sits in the chain's receiver (at any nesting depth) and the chain's own tail is a
plain property or index read rather than another call -- e.g. `trigger().x`, or
`trigger().inner.x`. Today the only in-repo consumer is this crate's own
`tests/example_convention_test.rs::example_scripts_follow_declarative_top_level_convention`,
which checks every `examples/scene_script/*/src/*.rhai` script -- not (yet) a gate applied to
arbitrary untrusted script content.

**What breaks:** Silent. `check_top_level_is_declarative()` returns `Ok( () )` for a script that
violates its own documented contract (`docs/invariant/001_top_level_bindings_convention.md`:
"only declarative bindings and a single trailing entry-point call are allowed at top level").
No error, no warning, no test failure -- the exact opposite of `docs/invariant/001`'s own claimed
Violation Consequences: enforcement is supposed to be "structural and loud, not a silent runtime
surprise".

**Magnitude:** 1 shared traversal helper (`call_in_expr()`'s `Expr::Dot` arm) -- the sole
call-detection path for every dotted/chained expression the checker inspects; every top-level
dot-chain statement passed to `call_expr()` went through it.

**Entity Scope:** None -- a code-level defect, not an OPS Entity concern.

**Which of the 3 documented checker gaps this fix closes:**
`docs/pitfall/002_checker_is_structural_not_semantic.md` documents 2 gaps: (1) a `let`
initializer's own content is never inspected (`Stmt::Var` is unconditionally `Role::Binding`);
(2) a call nested inside a larger expression's *arguments* is invisible when the outermost node
is itself a call (e.g. the operator call in `not_main() + 1`). This bug is a **3rd,
previously-undocumented gap**, distinct from both: `call_expr()` already had *explicit*
recursion into `Expr::Dot` nodes (unlike gap 2, where no recursion into a call's own arguments
happens at all) -- it just recursed into only one side (`.rhs`) of that `Dot`, never falling back
to the other (`.lhs`). This fix closes **only this 3rd gap**. Confirmed empirically this session
(temporary probe, since removed): `"fn trigger() { #{ x: 1 } } let y = trigger().x; y"` -- the
same receiver-call shape, wrapped in a `let` -- still returns `Ok( () )` both before AND after
this fix (gap 1, the `let`-initializer gap, untouched). Gap 2 (operator-argument nesting, e.g.
`not_main() + 1`) is architecturally untouched by this fix since `+` is `Expr::FnCall` directly
and never reaches `call_expr()`'s `Expr::Dot` arm at all. Gap 1 and gap 2 both remain fully open.

## How Discovered

A prior investigation pass in this session identified `call_expr()`'s one-sided `Expr::Dot`
recursion as a candidate gap and reported it for filing, fixing, and verification (this pass).
Re-confirmed directly and empirically in this session before any fix was applied:

```bash
cd module/helper/scene_script && cargo test -p scene_script --test example_convention_test -- --nocapture
```

(full pre-fix transcript captured this session):

```
thread 'checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read' (2960736) panicked at module/helper/scene_script/tests/example_convention_test.rs:226:6:
a call sitting in a dot chain's receiver must be rejected even when the chain's own tail is a plain property read: ()
...
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Also independently confirmed via direct source inspection that the sibling checker
`check_whole_ast_is_pure()` (`src/purity_lint.rs:67-91`) was never susceptible to this particular
gap: it delegates traversal to `rhai::AST::walk()` with its own independent `call_in_node()`
(`src/purity_lint.rs:37-46`) that pattern-matches `Stmt::FnCall`/`Expr::FnCall`/`Expr::MethodCall`
directly against whatever node `walk()` visits -- it has no `Dot`-specific one-sided recursion of
its own, so both sides of any dot chain are inherently visited.

## Minimum Reproducible Example

```rhai
fn trigger() { #{ x: 1 } }
trigger().x
```

`trigger()` is a real, non-`main` function call in the chain's receiver; `.x` (the chain's own
tail) is a plain property read, not a call. `check_top_level_is_declarative()` must reject this
script.

Pre-fix: returns `Ok( () )` -- the call is silently missed.
Post-fix: returns `Err( ImperativeTopLevelStatement { position: 1:28, kind: "expression" } )`.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/scene_script && cargo test -p scene_script --test example_convention_test checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|-------|---------|----------|
| H1 | `call_in_expr()`'s `Expr::Dot` arm recurses only into `.rhs`, never falling back to `.lhs`, so a call sitting in a chain's receiver with a non-call tail is never found. | ✅ Root Cause | Direct read of `top_level_lint.rs` (pre-fix, line 100): `rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ),` -- no `.lhs` fallback anywhere. | E1, E2 |
| H2 | Because no call is found, `role()` misclassifies the whole statement as `Role::PlainExpression`, which is allowed at any top-level position. | ✅ Verified | `role()`'s fallback arm (line 136 post-fix): `( rhai::Stmt::Expr( .. ), None ) => Role::PlainExpression`, reached whenever `call_expr()` returns `None`. | E1, E3 |
| H3 | The gap is confined to the dot-chain traversal and does not close either of the two gaps already documented in `docs/pitfall/002_checker_is_structural_not_semantic.md` (`let` initializer; call nested in operator argument). | ✅ Verified | `let y = trigger().x; y` reproduces `Ok( () )` both pre- AND post-fix -- `Stmt::Var` classifies `Role::Binding` unconditionally (line 133), never consulting `call_expr()`'s result at all, so this fix cannot affect it. | E4 |
| H4 | The sibling checker `check_whole_ast_is_pure()` (`src/purity_lint.rs`) was never susceptible to this gap in the first place, since it doesn't share `call_expr()`'s hand-rolled one-sided recursion. | ✅ Verified | Direct read of `purity_lint.rs:37-46,67-91`: `check_whole_ast_is_pure()` delegates to `rhai::AST::walk()` with its own `call_in_node()` matcher -- no `Expr::Dot`-specific traversal of its own to be one-sided in. | E5 |
| H5 | No currently-tracked example script (`examples/scene_script/*/src/*.rhai`) exercises the affected shape, so the fix carries no regression risk against real content. | ✅ Verified | Direct, full read of both tracked example scripts; neither has a top-level dot/index chain with a call in the receiver and a non-call tail. `example_scripts_follow_declarative_top_level_convention` passes both pre- and post-fix. | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/scene_script/src/top_level_lint.rs` (pre-fix, line 100) | `rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ),` -- single-sided recursion, no `.lhs` fallback. | H1 ✅ |
| E2 | `module/helper/scene_script/src/top_level_lint.rs:116` (post-fix) | `rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ).or_else( \|\| call_in_expr( &binary.lhs ) ),` -- confirms both the root cause (what was missing) and the fix (what closes it). | H1 ✅ |
| E3 | `module/helper/scene_script/src/top_level_lint.rs:136` | `( rhai::Stmt::Expr( .. ), None ) => Role::PlainExpression` -- the fallback `role()` arm reached whenever `call_expr()` finds no call. | H2 ✅ |
| E4 | `-0001_longrun.log`, `-0004_longrun.log` (this session, temporary probe test, since removed) | `PROBE let-wrapped result: Ok(())` for `"fn trigger() { #{ x: 1 } } let y = trigger().x; y"`, identical both pre-fix (`-0001`, line 41) and post-fix (`-0004`, line 10). | H3 ✅ |
| E5 | `module/helper/scene_script/src/purity_lint.rs:37-46,67-91` | `check_whole_ast_is_pure()` uses `ast.walk()` plus its own independent `call_in_node()` -- no `Dot`-specific recursion to be incomplete in. | H4 ✅ |
| E6 | `examples/scene_script/f32x2_vector_arithmetic/src/f32x2_vector_arithmetic.rhai`, `examples/scene_script/pingpong_animation/src/pingpong_animation.rhai` (direct, full read) | Neither script has a top-level dot/index chain with a call in the receiver and a non-call tail. | H5 ✅ |

## Root Cause

`call_expr()`'s inner `call_in_expr()` helper resolves a dotted/chained expression's call by
recursing through `Expr::Dot( BinaryExpr { lhs, rhs }, .. )` nodes. Chained dots nest with the
receiver as `lhs` and the next step as `rhs` (`a.b().c()` is `Dot( Dot( a, b() ), c() )`), so for
a chain whose own tail IS a call, recursing into `rhs` alone correctly reaches the terminal call.
But when the chain's own tail is a plain property or index read instead (`trigger().x` -- `rhs`
is the property `x`, `lhs` is the call `trigger()`), `rhs`-only recursion finds no call at all,
and the caller (`role()`) falls through to `Role::PlainExpression` rather than
`Role::Call( "trigger" )`.

## Why Not Caught

The only pre-existing dotted-call test, `checker_rejects_a_trailing_non_main_method_call`, puts
the call in the chain's own tail (`t.update( 0.5 )`) -- exactly the one shape `rhs`-only
recursion already handled correctly. No existing test exercised a call sitting in the chain's
*receiver* instead, so this shape's misclassification had no test coverage.

## Fix Location

`module/helper/scene_script/src/top_level_lint.rs`: `call_in_expr()`'s `Expr::Dot` arm (line
116) now falls back to `&binary.lhs` via `.or_else()` whenever recursing into `&binary.rhs` finds
no call:

```rust
// before (pre-fix, was line 100)
rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ),

// after (post-fix, line 116)
rhai::Expr::Dot( binary, .. ) => call_in_expr( &binary.rhs ).or_else( || call_in_expr( &binary.lhs ) ),
```

Both sides are checked at every `Dot` level (not just the outermost), so an arbitrarily deep
chain resolves correctly regardless of which side carries the one real call. The doc comment
above `call_expr()` (lines 81-97) and the `Fix(BUG-351)`/`Root cause`/`Pitfall` 3-field comment
block (lines 100-110) were updated accordingly.

## Prevention

`tests/example_convention_test.rs::checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`
(lines 194-228) pins the exact reported shape (`trigger().x`) as a permanent regression guard,
asserting `check_top_level_is_declarative()` now returns `Err` with `kind == "expression"`.

## Generalized Version

**Broken assumption:** "a hand-rolled AST traversal that already special-cases one recursive node
type (here, `Expr::Dot`) only needs to walk the side that happens to match the traversal's most
common/expected shape."

**Confirmed general rule:** A Rhai `Expr::Dot` chain (or any binary-shaped AST node with
`lhs`/`rhs`) can carry the content a traversal is searching for on EITHER side, not only the side
that happens to match the traversal's most-anticipated shape (a call chain's terminal call,
`rhs`) -- a receiver-position call with a non-call tail (`lhs`, e.g. `trigger().x`,
`trigger()[0]`, `trigger().a.b`) is an equally valid Rhai program shape and must be checked too.
Any future hand-rolled (non-`AST::walk`-based) traversal added to this checker must check both
sides of every binary node it special-cases, not assume the call-bearing side.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found via a prior investigation pass this session; re-confirmed directly via `cargo test -p scene_script --test example_convention_test`, pre-fix panic transcript captured in this session's `-0001_longrun.log`. |
| 2026-08-18 | fixed | `call_in_expr()`'s `Expr::Dot` arm (`src/top_level_lint.rs:116`) now falls back to `&binary.lhs` via `.or_else()` whenever `&binary.rhs` yields no call. |
| 2026-08-18 | VERIFY Gate | Reproducer test `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read` confirmed passing (`cargo test -p scene_script --test example_convention_test checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`: 1 passed; 0 failed); fix in `module/helper/scene_script/src/top_level_lint.rs:116` confirmed present (`.or_else( || call_in_expr( &binary.lhs ) )` fallback). state: Unverified -> Verified |

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/scene_script/src/top_level_lint.rs` | `call_in_expr()`'s `Expr::Dot` arm (line 116) now falls back to `.lhs` when `.rhs` yields no call (`Fix(BUG-351)` 3-field comment, lines 100-110); doc comment above `call_expr()` (lines 81-97) corrected to describe both sides being checked. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/scene_script/tests/example_convention_test.rs` | Added `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read` (lines 194-228, `bug_reproducer(BUG-351)`, 5-section doc comment). |

## Refs: docs/

| File | Change |
|------|--------|
| `module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md` | Corrected the `rhs`-only claim (lines 39-49): a dot chain's call can sit in either `lhs` or `rhs`, not `rhs` alone; added `BUG-351` backreference comment. |
| `module/helper/scene_script/docs/algorithm/001_top_level_statement_classification.md` | Corrected step 1's `call_expr()` description (line 20) to include the `.lhs` fallback; added `BUG-351` backreference comment (lines 21-22). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass found no placeholder sections; cross-checked the `**Related Bugs:** None` claim against `docs/pitfall/002_checker_is_structural_not_semantic.md`'s two documented gaps (let-initializer content, operator-argument nesting) — confirmed this is a genuine 3rd, distinct gap, not overlapping either | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass executed the crate's full test suite fresh (`cargo nextest run -p scene_script --all-features`, detached via longrun): 58/58 passed, including the exact reproducer `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`; `cargo test -p scene_script --doc --all-features`: 0/0 (crate carries no doctests, trivially clean). Adversarial pass read the actual fix diff directly (line 116, `.or_else( \|\| call_in_expr( &binary.lhs ) )`) rather than trusting the MRE's prose description, and independently re-derived that the pre-fix `trigger().x` case would return `Ok(())` (rhs=`x` is not a call, no lhs fallback existed) — matches the documented pre-fix Actual block exactly | — |
| D3 | Cross-Reference Integrity | — | 🟢 | 5 Hypothesis rows (H1 marked Root Cause), all H/E rows cross-cited and bidirectional; `grep -n "BUG-351"` across `src/top_level_lint.rs`, `tests/example_convention_test.rs`, and both `docs/invariant/001` and `docs/algorithm/001` confirms backreferences matching all 3 `## Refs:` sections. Adversarial pass independently re-verified H4 (sibling checker `check_whole_ast_is_pure()` never shared this gap) by reading `purity_lint.rs:37-91` directly — confirmed it delegates to `ast.walk()` with its own `call_in_node()` matcher, no `Dot`-specific one-sided recursion to be incomplete in; also independently re-verified H5 by reading both tracked `.rhai` example scripts in full — neither exercises the `call().property` shape | — |
| D4 | Root Cause Quality | — | 🟢 | Root Cause section traces exactly to the `.rhs`-only recursion gap, matching H1 and the current source at line 116. `## Fix Location` gives precise, current file:line. Adversarial pass checked the `.or_else()` fallback for unbounded-recursion risk on deeply nested chains — both branches terminate at `_ => None` for non-`Dot`/non-call nodes, so recursion depth is strictly bounded by AST depth, no infinite-loop risk introduced | — |
| D5 | Execution Scope | — | 🟢 | `repo_identity: self`; fix resolves inside `module/helper/scene_script/src/top_level_lint.rs`, same repo | — |
| D6 | Crate Scope Unity | — | 🟢 | `**Component:**` (`module/helper/scene_script`) matches the crate `## Fix Location` resolves to | — |
| D7 | Crate Locality | — | 🟢 | `scene_script` is the leaf crate directly owning `top_level_lint.rs` — not a pushed-up aggregator | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix stays within `scene_script`'s existing Rhai-glue/lint responsibility; no scope expansion | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced:** YES — `cargo nextest run -p scene_script --all-features` exit 0, 58/58 passed (includes `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`); `cargo test -p scene_script --doc --all-features` exit 0, 0/0 (no doctests in this crate), 2026-08-18.
