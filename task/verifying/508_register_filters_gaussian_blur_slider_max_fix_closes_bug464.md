# 508: Register filters gaussian blur slider max fix closes BUG-464

## Execution State

- **id:** 508
- **title:** Register filters gaussian blur slider max fix closes BUG-464
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-20 14:16:35
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/filters
- **closes:** BUG-464
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-20 14:21:07
- **expires_at:** 2026-08-20 16:21:07
- **unverified_at:** 2026-08-20 14:20:52
- **unverified_by:** unknown
- **verifying_at:** 2026-08-20 14:21:07
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

BUG-464 (`task/bug/verified/464_filters_gaussian_blur_slider_max_disproportionate_kernel_size.md`,
Medium severity, 🎯 Verified) found `examples/minwebgl/filters`'s Gaussian Blur slider allowing
`u_sigma` up to 50.0, which the shader's `kernel_size = u_sigma * 6 + 1` formula turns into a
301-sample-wide worst-case kernel -- wildly disproportionate to the Box (80) and Stack (161) blur
variants' own worst cases at their own slider maxima, and expensive enough on a large sigma to make
the demo visibly stutter or hang the GPU pipeline momentarily. The fix -- lowering the Gaussian
slider's max from 50.0 to 15.0 in `blur_filters_setup` (worst-case kernel_size 91, now in line with
the other two variants) -- is already applied and documented with a `Fix(BUG-464)`/`Root
cause`/`Pitfall` comment in `src/ui_setup/filter_setup_advanced.rs`. This task performs the
remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to
Task` (PROC12) -- to formally register that already-complete, already-verified fix as a tracked
task, closing BUG-464.
Testable: `cd examples/minwebgl/filters && cargo check --target wasm32-unknown-unknown` → exit 0.

## In Scope

- `examples/minwebgl/filters/src/ui_setup/filter_setup_advanced.rs`'s already-applied slider-max
  reduction (50.0 → 15.0) in the Gaussian Blur variant's `blur_filter_setup` call within
  `blur_filters_setup` -- verify present; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/464_filters_gaussian_blur_slider_max_disproportionate_kernel_size.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `examples/minwebgl/filters` -- the fix is complete and independently
  verified by the bug's own VERIFY Gate.
- Re-running BUG-464's own VERIFY Gate -- already run and recorded in the bug file's Verification
  Record; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- Rebalancing the Box or Stack blur variants' own slider maxima -- BUG-464 only flagged the
  Gaussian variant's specific disproportion (301 vs 80/161); the other two were not found
  disproportionate to each other and are untouched by this fix.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Fix already applied: Gaussian Blur's slider max lowered from `50.0` to `15.0` in
  `blur_filters_setup`'s call into `filter_setup_helpers::blur_filter_setup` for the
  `blur::Gaussian` variant.
- Green state already confirmed, and re-confirmed live during this task's filing:
  `cargo check -p filters --target wasm32-unknown-unknown` compiles clean.
- No refactor needed -- the fix is a single numeric literal change plus its documentation comment.
- Fix documentation already complete at the bug level: BUG-464 carries the full Root Cause/Why Not
  Caught/Fix Location/Prevention narrative in its own body, including the worst-case kernel_size
  arithmetic for all 3 blur variants -- this task does not duplicate it, only cross-links via
  `closes: BUG-464`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention -- document rather than force/spoof if
  so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -n "Fix(BUG-464)" examples/minwebgl/filters/src/ui_setup/filter_setup_advanced.rs` | Whole-file scan for the fix comment | present at the Gaussian Blur setup site |
| T02 | `grep -n "blur::Gaussian" examples/minwebgl/filters/src/ui_setup/filter_setup_advanced.rs` | Locate the Gaussian variant's setup call | slider max argument reads `15.0`, not `50.0` |
| T03 | Hand-compute `u_sigma * 6 + 1` at the new max | `u_sigma = 15.0` | `kernel_size = 91`, in line with Box (80) / Stack (161) worst cases, not the pre-fix 301 |
| T04 | `cd examples/minwebgl/filters && cargo check --target wasm32-unknown-unknown` | crate compiles for wasm32 | 0 errors |

## Acceptance Criteria

- The Gaussian Blur variant's slider-max argument in `blur_filters_setup` reads `15.0`
- The fix's source comment carries all 3 required fields: `Fix(BUG-464)`, `Root cause`, `Pitfall`
- `cargo check -p filters --target wasm32-unknown-unknown` succeeds
- `task/bug/verified/464_filters_gaussian_blur_slider_max_disproportionate_kernel_size.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify -- an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 -- Does the Gaussian Blur variant's slider-max argument read `15.0` in
  `blur_filters_setup`?
- [ ] C2 -- Does `u_sigma * 6 + 1` at `u_sigma = 15.0` evaluate to a worst-case `kernel_size` of 91,
  in line with the Box/Stack variants' own worst cases?
- [ ] C3 -- Does the fix's source comment carry `Fix(BUG-464)`, `Root cause`, and `Pitfall` fields?
- [ ] C4 -- Does `cargo check -p filters --target wasm32-unknown-unknown` succeed with 0 errors?

**Registration correctness**
- [ ] C5 -- Does this task's `closes:` field name `BUG-464`?
- [ ] C6 -- Does BUG-464's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 -- No Edit/Write tool call in this task's own execution touched the Box or Stack blur
  variants' own slider-max values (the fix content matches what BUG-464's own already-completed
  fix applied, Gaussian-only; this task made no further source edit -- note this repo's working
  tree carries many pre-existing, unrelated uncommitted changes from other concurrent activity, so
  a blanket repo-wide `git diff --stat` is not a meaningful signal here).

### Measurements

- [ ] M1 -- `grep -n "15.0\|50.0" examples/minwebgl/filters/src/ui_setup/filter_setup_advanced.rs`
  → the Gaussian Blur call site's slider-max literal reads `15.0`, and no remaining `50.0` literal
  is wired to the Gaussian variant

### Invariants

- [ ] I1 -- `cargo check -p filters --target wasm32-unknown-unknown` → 0 errors

### Anti-faking checks

- [ ] AF1 -- the changed literal is genuinely reached by the Gaussian Blur variant's own code path
  (not a similarly-named but unrelated constant, and not a value that's immediately overridden
  elsewhere before reaching the slider widget) -- checked by reading `blur_filters_setup`'s call
  chain into `filter_setup_helpers::blur_filter_setup`, not just grepping for the literal in
  isolation

## Verification Record

**Gate Round 1** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | In Scope names the single already-applied slider-max change; Out of Scope excludes rebalancing Box/Stack's own maxima (not flagged by BUG-464). Adversarial pass: checked whether Box (80.0) and Stack (80.0 argument, 161 worst-case per the bug's own arithmetic) should also have been touched — re-read BUG-464's own body, confirmed it explicitly scopes to Gaussian only ("Gaussian's own specific disproportion"), so leaving the other two untouched is correct, not an omission. | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (cites BUG-464 path/severity/state), Observable (names the exact literal change, 50.0→15.0), Scoped (registration only), Testable (wasm32 compile command). Adversarial pass: checked whether "Testable" should instead assert the kernel_size arithmetic itself (91 vs 301) rather than just compiling — confirmed T03 already covers that hand-computation separately in the Test Matrix, so the MOST line's compile-only Testable claim is appropriately narrow, not incomplete. | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: "does this need a tracked task?" — yes, same PROC12 registration requirement as every other promoted bug this round. No speculative rebalancing of unrelated sliders added. | — |
| D4 | Implementation Readiness | — | 🟢 | Delivery Requirements state the fix is already applied; Test Matrix T01-T04 all executable. Adversarial pass: ran all 4 for real this round — T01/T02 grep hits confirmed at exact lines, T03's hand-computation independently re-derived (`15*6+1=91`), T04 (`cargo check -p filters --target wasm32-unknown-unknown` via longrun, shared launch also covering task 507) returned exit 0. | — |
| D5 | Execution Scope | — | 🟢 | Touched file (`filter_setup_advanced.rs`) resolves inside this repository, under `examples/minwebgl/filters/src/ui_setup/`. | — |
| D6 | Crate Scope Unity | — | 🟢 | Every deliverable path resolves inside exactly one crate (`filters`) — same crate as task 507, but each task's own In/Out of Scope stays independently coherent (507 touches `renderer.rs`/`framebuffer.rs`, 508 touches `ui_setup/filter_setup_advanced.rs`; zero file overlap between the two). | — |
| D7 | Crate Locality | — | 🟢 | Fix and task both target the leaf crate that owns the defect (`filters`), specifically its own `ui_setup` submodule where the slider wiring lives. | — |
| D8 | Crate Single Responsibility | — | 🟢 | `filters` crate's responsibility stays statable without "and"; this task's registration work doesn't expand it. | — |

**Live re-verification (this round, not carried over from the bug's own VERIFY Gate):**
- `grep -n "Fix(BUG-464)" src/ui_setup/filter_setup_advanced.rs` → present at line 86, full Root cause/Pitfall comment through line 94.
- `grep -n "Gaussian" -B2 -A2 src/ui_setup/filter_setup_advanced.rs` → confirmed the call site at line 105: `blur_filter_setup( filter_renderer, current_filter, "gaussian-blur", blur::Gaussian, 15.0 )`, alongside Box (line 104, `80.0`) and Stack (line 106, `80.0`) unchanged, matching BUG-464's own scope.
- Hand-recomputed `u_sigma * 6 + 1` at `u_sigma = 15.0` → `91`, matching the bug file's own claimed worst case and the task's T03 expectation.
- `longrun .launch -- cargo check -p filters --target wasm32-unknown-unknown` → exit 0, 45s elapsed (same launch also verified task 507's files in the same crate), no errors or warnings.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | FILED | Task created via PROC12 (bug_promote) for BUG-464. |
| 2026-08-20 | READINESS_GATE_PASS | 8/8 dimensions 🟢 on live re-verification; task claimed for verification via `tsk .claim_verify 508`. |
| 2026-08-20 | EXECUTED | Fix was already applied prior to this task's filing (BUG-464's own fix); this task's execution is the registration/verification walk itself, confirmed complete. |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-20 14:16:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FILED | task created |
| 2026-08-20 14:23:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | same-actor guard expected to block (filed_by == actor); documenting attempt per project convention |
| 2026-08-20 14:20:52 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-20 14:21:07 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
