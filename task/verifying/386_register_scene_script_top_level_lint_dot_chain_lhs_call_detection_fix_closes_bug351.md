# 386: Register scene_script top level lint dot chain lhs call detection fix closes BUG-351

## Execution State

- **id:** 386
- **title:** Register scene_script top level lint dot chain lhs call detection fix closes BUG-351
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:48:40
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/scene_script/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/scene_script
- **closes:** BUG-351
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 23:49:14
- **expires_at:** 2026-08-19 01:49:14
- **unverified_at:** 2026-08-18 23:47:43
- **unverified_by:** system
- **verifying_at:** 2026-08-18 23:49:14
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-351 (`task/bug/verified/351_top_level_lint_misses_call_in_dot_chain_property_tail.md`,
Medium severity, 🎯 Verified) found `module/helper/scene_script`'s
`top_level_lint.rs::call_expr()` helper `call_in_expr()` recursed into a Rhai `Expr::Dot`
chain's `.rhs` only, silently missing a real, non-`main` call sitting in the chain's receiver
(`.lhs`) whenever the chain's own tail was a plain property/index read instead of another call
(e.g. `trigger().x`) — the statement was misclassified `Role::PlainExpression` (allowed anywhere
at top level) instead of `Role::Call( "trigger" )` (rejected), silently defeating the exact
declarative-top-level safety net `docs/invariant/001` documents. The fix — `call_in_expr()`'s
`Expr::Dot` arm now falls back to `&binary.lhs` via `.or_else()` whenever `&binary.rhs` yields no
call, checking both sides at every nesting level — is already applied
(`src/top_level_lint.rs:116`) with the required `Fix(BUG-351)`/`Root cause`/`Pitfall` 3-field
comment, and independently confirmed via a new reproducer test
(`checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`,
`tests/example_convention_test.rs:215`) plus corrected cross-references in
`docs/invariant/001_top_level_bindings_convention.md` and
`docs/algorithm/001_top_level_statement_classification.md` — the bug file's own Verification
Record, 8/8 PASS, 2026-08-18 (`cargo nextest run -p scene_script --all-features`: 58/58 passed).
This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-351.
Testable: `cd module/helper/scene_script && cargo test -p scene_script --test
example_convention_test checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`
→ 1 passed (bug file's own recorded evidence; this task's own filing-time attempt hit an
external, unrelated workspace build blocker — see Verification Record D2 for full attribution).

## In Scope

- `module/helper/scene_script/src/top_level_lint.rs` — the already-applied `call_in_expr()`
  `Expr::Dot` arm `.lhs` fallback (line 116) and its `Fix(BUG-351)`/`Root cause`/`Pitfall` source
  comment (lines 100-110) — verify present via direct read; no further edit expected.
- The already-applied
  `tests/example_convention_test.rs::checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`
  reproducer (lines 215-228) — verify present via direct read; no further edit expected.
- The already-applied doc corrections in `docs/invariant/001_top_level_bindings_convention.md`
  and `docs/algorithm/001_top_level_statement_classification.md` — verify `BUG-351`
  backreference present in both; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking
  `task/bug/verified/351_top_level_lint_misses_call_in_dot_chain_property_tail.md`'s header back
  to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/scene_script` — the fix is complete and verified by
  the bug's own Verification Record (8/8 PASS, 2026-08-18).
- Re-running or amending BUG-351's own Verification Record — already run and recorded in the bug
  file; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- The two still-open, previously-documented checker gaps
  (`docs/pitfall/002_checker_is_structural_not_semantic.md`: a `let` initializer's own content is
  never inspected; a call nested inside a larger expression's arguments is invisible) — the bug
  file's own Impact section confirms this fix closes only the 3rd, dot-chain-receiver gap;
  fixing either of the other two is a separate, not-yet-filed concern, not part of this
  registration task.
- `src/purity_lint.rs`'s `check_whole_ast_is_pure()` — the bug file's own H4/E5 confirmed it was
  never susceptible to this gap (independent `ast.walk()`-based traversal); not touched by this
  fix or this task.
- Diagnosing or fixing the external `mdmath_core`/`ndarray_cg` workspace build blocker
  encountered during this task's own filing (see Verification Record D2) — a concurrent,
  unrelated in-flight refactor in a different crate family (`module/math/`), entirely outside
  `scene_script`'s and this task's remit; documented for transparency only.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own Symptom section directly captured
  the pre-fix panic (`expect_err` receiving `Ok(())`) via `cargo test -p scene_script --test
  example_convention_test -- --nocapture`.
- Fix already applied: `call_in_expr()`'s `Expr::Dot` arm now checks `.rhs` first, falling back
  to `.lhs` via `.or_else()`, with the required 3-field source comment.
- Green state already confirmed by the bug file's own Verification Record (2026-08-18): `cargo
  nextest run -p scene_script --all-features` → 58/58 passed, includes the reproducer. This
  task's own filing-time attempt to re-confirm live hit an external, unrelated blocker
  (documented in Verification Record D2) — the bug-level evidence stands independently.
- No refactor needed — the fix is a single `.or_else()` fallback added to one match arm, no
  structural churn.
- Fix documentation already complete at the bug level: BUG-351 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention/Generalized Version narrative, plus doc-level corrections in
  `docs/invariant/001` and `docs/algorithm/001` — this task does not duplicate it, only
  cross-links via `closes: BUG-351`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|--------------------|
| T01 | `cargo test -p scene_script --test example_convention_test checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read` | reproducer test | 1 passed (per bug file's own recorded evidence; blocked live at this task's filing time by an external, unrelated workspace issue — see Verification Record D2) |
| T02 | `check_top_level_is_declarative()` on `"fn trigger() { #{ x: 1 } } trigger().x"` | fixed `call_in_expr()` | `Err( ImperativeTopLevelStatement { kind: "expression", .. } )` |
| T03 | `grep -c "Fix(BUG-351)" src/top_level_lint.rs` | fix comment present | 1 |
| T04 | `grep -c "BUG-351"` across `docs/invariant/001_top_level_bindings_convention.md` and `docs/algorithm/001_top_level_statement_classification.md` | doc backreferences present | 1, 1 |
| T05 | `cargo nextest run -p scene_script --all-features` | full crate suite | 58/58 passed (per bug file's own recorded evidence, 2026-08-18) |

## Acceptance Criteria

- `module/helper/scene_script/src/top_level_lint.rs`'s `call_in_expr()` `Expr::Dot` arm checks
  `.rhs` then falls back to `.lhs` via `.or_else()`
- The fix carries all 3 required source-comment fields: `Fix(BUG-351)`, `Root cause`, `Pitfall`
- `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read` exists in
  `tests/example_convention_test.rs` and passes
- Both `docs/invariant/001` and `docs/algorithm/001` carry a `BUG-351` backreference
- `task/bug/verified/351_top_level_lint_misses_call_in_dot_chain_property_tail.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row's claim holds against either a live run or the bug file's own recorded
  evidence (whichever this task's own filing-time environment allowed)

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `call_in_expr()`'s `Expr::Dot` arm read `call_in_expr( &binary.rhs ).or_else( ||
  call_in_expr( &binary.lhs ) )` (line 116)?
- [ ] C2 — Does the fix's source comment carry `Fix(BUG-351)`, `Root cause`, and `Pitfall`
  fields?
- [ ] C3 — Does `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read` exist
  in `tests/example_convention_test.rs` and construct the real `"fn trigger() { #{ x: 1 } }
  trigger().x"` script via the real `check_top_level_is_declarative()` call (not a hardcoded
  expected-error literal)?
- [ ] C4 — Does `cargo nextest run -p scene_script --all-features` (via `longrun`, when the
  workspace build is not externally blocked) pass 58/58?
- [ ] C5 — Do both `docs/invariant/001` and `docs/algorithm/001` carry a `BUG-351`
  backreference?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-351`?
- [ ] C7 — Does BUG-351's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/scene_script/src/top_level_lint.rs`, its test file, its two doc files, or any
  `module/math/` file (the mdmath_core/ndarray_cg blocker is diagnosed, not fixed, by this task)
  — note this repo's working tree carries many pre-existing, unrelated uncommitted changes from
  other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here.

### Measurements

- [ ] M1 — `grep -c "Fix(BUG-351)" src/top_level_lint.rs` → 1
- [ ] M2 — `grep -c "BUG-351"` across `docs/invariant/001_top_level_bindings_convention.md`,
  `docs/algorithm/001_top_level_statement_classification.md` → 1, 1

### Invariants

- [ ] I1 — When the workspace build is not externally blocked: `cargo nextest run -p
  scene_script --all-features` → 0 failures

### Anti-faking checks

- [ ] AF1 — the reproducer test actually compiles the real Rhai script `"fn trigger() { #{ x: 1
  } } trigger().x"` via `engine.compile(..)` and calls the real
  `check_top_level_is_declarative(&ast)` (not a hardcoded expected-error literal standing in for
  the call) — checked by reading the test body itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass confirmed In/Out Scope enumerate all 4 touched files (source, test, 2 docs) matching the bug's own `## Refs:` sections exactly; confirmed the 2 still-open sibling checker gaps (`docs/pitfall/002`) are correctly named Out of Scope, not silently conflated with this fix. | — |
| D2 | MOST Goal Quality | — | 🟢 | Confirming pass relied on the bug file's own recorded Verification Record (`cargo nextest run -p scene_script --all-features` → 58/58 passed, 2026-08-18). Adversarial pass attempted a fresh live re-run during this task's own filing and hit the SAME external blocker discovered and root-caused during BUG-350/task 385's gate (this session, ~20 minutes earlier): `ndarray_cg`'s `reuse ::mdmath_core::general;` broken by a concurrent `mdmath_core` refactor (mtime 20:38:31) that removed/renamed the `general` layer. `cargo tree -p scene_script -i ndarray_cg` confirms `scene_script` depends on it transitively (`scene_script → animation → mingl → ndarray_cg`), so this crate is equally affected — same root cause, not re-investigated from scratch, cross-referenced to task 385's D2 for the full mtime/attribution evidence. Does not implicate BUG-351's own fix, which was independently re-confirmed via direct, full-body source reads of both the fix (`top_level_lint.rs:100-127`) and the reproducer test (`example_convention_test.rs:194-228`) during this task's own investigation — both match the bug file's claims character-for-character. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | `grep -c "BUG-351"` (live, this gate) across all 4 touched files: `src/top_level_lint.rs`:1, `tests/example_convention_test.rs`:1, `docs/invariant/001_top_level_bindings_convention.md`:1, `docs/algorithm/001_top_level_statement_classification.md`:1 — matches `## Refs:` exactly. | — |
| D4 | Root Cause Quality | — | 🟢 | Direct read of `call_in_expr()` (lines 111-119) confirms the `Expr::Dot` arm reads exactly `call_in_expr( &binary.rhs ).or_else( \|\| call_in_expr( &binary.lhs ) )` — matches `## Fix Location` exactly. Adversarial pass checked for unbounded-recursion risk on deeply nested chains: both `call_in_expr` branches terminate at `_ => None` for non-`Dot`/non-call nodes, so recursion depth is strictly bounded by AST depth — no risk introduced by the `.or_else()` fallback. | — |
| D5 | Execution Scope | — | 🟢 | `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/scene_script` matches the actual crate path; `-p scene_script` resolves to this package. | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`scene_script`) throughout In Scope/Out of Scope — the 2 touched docs files also live under `module/helper/scene_script/docs/`, not a second crate. | — |
| D7 | Crate Locality | — | 🟢 | Confirmed via live read that `top_level_lint.rs` and `example_convention_test.rs` physically live under `module/helper/scene_script/` — matches the `unit` field. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix stays within `scene_script`'s existing Rhai-glue/lint responsibility; confirmed `purity_lint.rs`'s independent `check_whole_ast_is_pure()` is untouched (bug file's own H4/E5) — no entanglement. | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced (bug file's own recorded evidence, 2026-08-18, predating this session's later
external breakage):** `cargo nextest run -p scene_script --all-features` → 58/58 passed,
includes `checker_rejects_a_trailing_call_disguised_as_a_dot_chain_property_read`. **This task's
own filing-time direct verification:** full-body read of `src/top_level_lint.rs:81-127`
(fix + doc comment) and `tests/example_convention_test.rs:194-228` (reproducer) both match the
bug file's claims exactly, character-for-character. **External blocker (informational, not a
task or fix defect, same root cause as task 385/BUG-350, not re-derived here):** a fresh live
re-run attempted 2026-08-18 ~20:49-20:50 hit `ndarray_cg`'s currently-broken
`reuse ::mdmath_core::general;` (root-caused in task 385's Verification Record D2); confirmed via
`cargo tree -p scene_script -i ndarray_cg` that `scene_script` transitively depends on it
(`scene_script → animation → mingl → ndarray_cg`), so this is the same workspace-wide condition,
not a new or scene_script-specific issue.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:48:40 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/scene_script/ | FILED | task created |
| 2026-08-18 20:49:53 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:49:53 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/scene_script/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/scene_script/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 386 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/scene_script/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:43 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-351's
  already-applied, already-verified fix (`module/helper/scene_script/src/top_level_lint.rs`'s
  `call_in_expr()` `Expr::Dot` arm now falls back to `.lhs` via `.or_else()` whenever `.rhs`
  yields no call, catching a real top-level call hidden in a dot chain's receiver, e.g.
  `trigger().x`) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Confirming pass relied on the bug file's own recorded Verification Record (58/58 passed).
  Adversarial pass independently re-read the fix and reproducer test bodies in full (matching
  exactly) and attempted a fresh live re-run, hitting the same external `mdmath_core`/
  `ndarray_cg` workspace blocker first diagnosed during task 385/BUG-350's own gate (cross-
  referenced, not re-derived); confirmed via `cargo tree` that `scene_script` shares the same
  transitive dependency path. Documented in full rather than silently retried or ignored; does
  not implicate BUG-351's own fix. `tsk .claim_verify 386` succeeded (❓→🔬, moved to
  `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and verified (bug file's own Verification Record,
  2026-08-18) during BUG-351's own investigation. This task's own contribution is the formal
  tracking registration and lifecycle walk, not the code change itself. `tsk .verify_pass 386`
  blocked by the same-actor guard (documented above) — task left at 🔬 Verifying per this
  sandbox's standing, previously documented limitation, not a quality defect in this task's own
  content.
