# Register canvas_renderer's framebuffer-restore fix (closes BUG-342)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 16:13:44
- **expires_at:** 2026-08-18 18:13:44
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-342
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/canvas_renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-18 16:13:44
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

BUG-342 (`task/bug/verified/342_canvas_renderer_render_framebuffer_not_restored.md`, Medium
severity, 🎯 Verified) found `CanvasRenderer::render` (`module/helper/canvas_renderer/src/renderer.rs`)
binding `self.framebuffer` at line 309 but never restoring the default (`None`) binding before
returning -- unlike its two siblings in the same file, `framebuffer_create` (line 78) and
`texture_set` (line 395), which both explicitly restore `None` as their last GL state change.
WebGL's `bindFramebuffer` state persists on the context after `render()` returns, so any future or
third-party caller that issues a GL call right afterward without itself rebinding a framebuffer
first would silently target the internal offscreen texture instead of the intended target -- latent
today only because all 3 real call sites happen to immediately chain a different renderer's own
`.render(...)` call that rebinds its own target first (masking the leak by luck of call-site
ordering, not by any restore `render` itself performs). The fix (adding
`gl.bind_framebuffer( GL::FRAMEBUFFER, None );` before the `Ok(())` return, with a
`Fix(BUG-342)`/`Root cause`/`Pitfall` 3-field source comment) is already applied, together with a
structural regression test (`module/helper/canvas_renderer/tests/renderer_test.rs::render_restores_default_framebuffer_binding_before_returning`,
which extracts `render`'s body verbatim from the real source at test-run time via brace-counting
and asserts the restore call is present after the `self.framebuffer` bind -- this crate has no live
`WebGl2RenderingContext` test infrastructure, the same precedent BUG-227 already established for
this exact crate), independently re-confirmed this filing session via a fresh
`cargo test -p canvas_renderer --all-features` run (2 passed, 1 doc-test passed, 0 failed). This
task performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core Procedures : Procedure
- Promote Bug to Task` (PROC12) -- to formally register that already-complete, already-verified fix
as a tracked task, closing BUG-342.
Testable: `grep -qF 'gl.bind_framebuffer( GL::FRAMEBUFFER, None );' module/helper/canvas_renderer/src/renderer.rs
&& cargo test -p canvas_renderer --all-features 2>&1 | grep -q '^test result: ok' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/canvas_renderer/src/renderer.rs` (now lines 358-374) -- the already-applied
  `gl.bind_framebuffer( GL::FRAMEBUFFER, None );` restore call at the end of `render`'s body, and
  its `Fix(BUG-342)`/`Root cause`/`Pitfall` source comment (verify both are present; no further edit
  expected).
- `module/helper/canvas_renderer/tests/renderer_test.rs` -- the already-added
  `render_restores_default_framebuffer_binding_before_returning` structural reproducer (verify
  present and passing; no further edit expected).
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/342_canvas_renderer_render_framebuffer_not_restored.md`'s header back
  to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/canvas_renderer` -- the fix is complete; the two
  sibling functions (`framebuffer_create`, `texture_set`) were confirmed already-correct by
  BUG-342's own investigation (Hypothesis H2), not touched by the fix.
- BUG-227 (same crate, same file -- `framebuffer_create`'s renderbuffer-creation panic) --
  independent defect, no shared code path, requiring its own separate lifecycle; BUG-342's own
  header explicitly notes it as a Related Bug, not a duplicate.
- Adding live-`WebGl2RenderingContext` (`wasm-bindgen-test`) infrastructure to this crate -- BUG-342's
  own MRE/Prevention sections already judged this disproportionate for a one-line restore-call fix,
  matching BUG-227's own precedent MAAV D2 cost/benefit pass for this exact crate.
- Any other function's `bind_framebuffer` restore contract outside `render` -- BUG-342's own
  Generalized Version section confirms `render` is the only function workspace-wide (via
  `grep -rn "bind_framebuffer.*Some" --include=*.rs .`) that binds a non-default framebuffer without
  a matching restore.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-342's own MRE -- running
    `render_restores_default_framebuffer_binding_before_returning` against the pre-fix source
    panicked with "next bind_framebuffer call found after the self.framebuffer bind was: None"
    (i.e., no restore call found)
-   Fix already applied: `module/helper/canvas_renderer/src/renderer.rs` states
    `gl.bind_framebuffer( GL::FRAMEBUFFER, None );` immediately before `render`'s `Ok(())` return,
    with the 3-field `Fix(BUG-342)`/`Root cause`/`Pitfall` source comment in place
-   Green state already confirmed: this task's own filing session re-ran
    `cargo test -p canvas_renderer --all-features` fresh (exit 0, 2 passed + 1 doc-test, 0 failed,
    via `longrun`, log `-0089_longrun.log`)
-   No refactor needed -- single-statement addition (one `bind_framebuffer` restore call plus its
    source comment), no structural churn
-   Fix documentation already complete at the bug level: BUG-342 carries the 5-section fix
    documentation (Root Cause, Why Not Caught, Fix Location, Prevention, Pitfall) in its own body;
    this task does not duplicate it, only cross-links via `closes: BUG-342`
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate;
    `tsk .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit
    this sandbox's known same-actor guard, per project convention -- document rather than force/
    spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo test -p canvas_renderer --all-features --test renderer_test render_restores_default_framebuffer_binding_before_returning -- --exact` | Fixed `render`'s regression coverage | exit 0, 1 passed |
| T02 | `grep -qF 'gl.bind_framebuffer( GL::FRAMEBUFFER, None );' module/helper/canvas_renderer/src/renderer.rs` | Fixed `render` body, restore call present | exit 0 (match found) |
| T03 | `grep -rn 'bind_framebuffer.*Some' --include=*.rs . \| grep -v test` (BUG-342's own repeat-defect detector) | Whole-workspace scan for a non-default framebuffer bind with no matching restore | only `render`'s own already-fixed call site (now paired with a restore), no other unmatched site |
| T04 | `cargo test -p canvas_renderer --all-features` | Whole crate, both existing tests | exit 0, 2 passed, 1 doc-test passed |

## Acceptance Criteria

-   `module/helper/canvas_renderer/src/renderer.rs` states
    `gl.bind_framebuffer( GL::FRAMEBUFFER, None );` immediately before `render`'s `Ok(())` return
-   The same call site's source comment carries all 3 required fields: `Fix(BUG-342)`,
    `Root cause`, `Pitfall`
-   `module/helper/canvas_renderer/tests/renderer_test.rs` contains
    `render_restores_default_framebuffer_binding_before_returning`, and it passes
-   `task/bug/verified/342_canvas_renderer_render_framebuffer_not_restored.md`'s header states
    `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify -- an independent verifier performs the
walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `module/helper/canvas_renderer/src/renderer.rs` state `gl.bind_framebuffer( GL::FRAMEBUFFER, None );` immediately before `render`'s `Ok(())` return?
- [ ] C2 — Does the same call site's source comment carry `Fix(BUG-342)`, `Root cause`, and `Pitfall` fields?
- [ ] C3 — Does `module/helper/canvas_renderer/tests/renderer_test.rs` contain `render_restores_default_framebuffer_binding_before_returning`?
- [ ] C4 — Does `cargo test -p canvas_renderer --all-features` pass (0 failed)?
- [ ] C5 — Does a repo-wide grep for an unmatched `bind_framebuffer(..., Some(...))` (no paired `None` restore in the same function) return empty outside `render`'s own already-fixed site?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-342`?
- [ ] C7 — Does BUG-342's own header carry a `**Fix Task:**` line pointing back at this task's ID?

**Out of Scope confirmation**
- [ ] C8 — Are `framebuffer_create` and `texture_set` (the library source's other two functions) untouched by this task (`git diff --stat` empty for any line outside `render`'s own body and the new test)?
- [ ] C9 — Is BUG-227 absent from this task's own scope (no code change addressing `framebuffer_create`'s renderbuffer-creation panic)?

### Measurements

- [ ] M1 — `grep -c 'gl.bind_framebuffer( GL::FRAMEBUFFER, None );' module/helper/canvas_renderer/src/renderer.rs` → 3 (was: 2, pre-fix -- `framebuffer_create` and `texture_set`'s own pre-existing restores, plus `render`'s new one)
- [ ] M2 — `grep -c 'fn render_restores_default_framebuffer_binding_before_returning' module/helper/canvas_renderer/tests/renderer_test.rs` → 1

### Invariants

- [ ] I1 — `framebuffer_create` and `texture_set` unaffected: their own existing `bind_framebuffer( ..., None )` restore calls (lines 78, 395) remain textually unchanged
- [ ] I2 — `canvas_renderer` crate still green: `cargo test -p canvas_renderer --all-features` → 0 failures

### Anti-faking checks

- [ ] AF1 — the fix adds only the restore call and its source comment, not a change to `render`'s existing `Some( &self.framebuffer )` bind or its draw logic -- checked by reading the literal diff, not just the presence of the new line

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 16:07:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created (ID 368; highest_id was 367 after an earlier misfired `tsk .create` invocation this session burned that ID on a stray draft, deleted before this filing) |
| 2026-08-18 16:13:44 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 16:14:02 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 368 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |

## History

- **[2026-08-18]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally register
  BUG-342's already-applied, already-verified fix
  (`module/helper/canvas_renderer/src/renderer.rs`, added the missing default-framebuffer restore
  call at the end of `render`) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` — Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS; 1
  non-blocking D3 finding (wildcard `227_*.md` Related Documentation link) fixed during the gate
  itself. State transition to 🎯 is asserted only by `tsk .verify_pass` succeeding (see below),
  never by a direct hand-edit of this field.
- **[2026-08-18]** `CLAIM_VERIFY` — `tsk .claim_verify 368 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"`
  succeeded (❓→🔬, moved to `verifying/`). No new code edit performed: the described fix
  (`module/helper/canvas_renderer/src/renderer.rs` default-framebuffer restore call,
  `Fix(BUG-342)`/`Root cause`/`Pitfall` comment, structural regression test) already existed on
  disk prior to this task's filing, applied during BUG-342's own investigation. This task's own
  contribution is the formal tracking registration and lifecycle walk, not the code change itself.
  `tsk .verify_pass 368` blocked by the same-actor guard (documented in `## Journal` above) — task
  left at 🔬 Verifying per this sandbox's standing, previously-documented limitation (same guard
  that blocked tasks 358, 359, and 366's own `.verify_pass`), not a quality defect in this task's
  own content.

## Verification Record

**VERIFY Gate (2026-08-18) — Tier 2 Dual-Role Self-Check, 8 dimensions, verdict: PASS (8/8).**

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Uses BUG-342's own precedented no-live-GL structural-test exception | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Related Documentation initially cited a wildcard `227_*.md` path | Resolved to real filename `task/bug/completed/227_canvas_renderer_framebuffer_create_renderbuffer_unwrap_panic.md` before this gate |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`canvas_renderer`) only | — |
| D7 | Crate Locality | — | 🟢 | Confirmed no nested `task/`/`-task/` registry under `module/helper/canvas_renderer` — repo-root `task/` is the correct registry | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 issue | 1 fix |

**Adversarial pass specifics:** independently re-ran `cargo test -p canvas_renderer --all-features` fresh (exit 0, 2 passed + 1 doc-test, 0 failed); recounted M1 (`grep -c` for the restore call) → 3, matching the task's own stated expectation; confirmed BUG-227's cross-reference resolves to a real file; confirmed no ID collision on 368 (`find task -name "368_*.md"` → exactly one match).

**Reproduced:** YES — `cargo test -p canvas_renderer --all-features`, exit 0, 2026-08-18 (`-0089_longrun.log`).

## Related Documentation

- `task/bug/verified/342_canvas_renderer_render_framebuffer_not_restored.md` — the source bug this
  task promotes; carries the full Root Cause/MRE/Prevention/History detail this task does not
  duplicate
- `task/bug/completed/227_canvas_renderer_framebuffer_create_renderbuffer_unwrap_panic.md` —
  Related Bug, same crate/file, independent defect, not addressed by this task
