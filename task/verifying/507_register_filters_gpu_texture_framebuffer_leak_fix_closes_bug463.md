# 507: Register filters GPU texture framebuffer leak fix closes BUG-463

## Execution State

- **id:** 507
- **title:** Register filters GPU texture framebuffer leak fix closes BUG-463
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-20 14:16:19
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/filters
- **closes:** BUG-463
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-20 14:21:07
- **expires_at:** 2026-08-20 16:21:07
- **unverified_at:** 2026-08-20 14:20:52
- **unverified_by:** unknown
- **verifying_at:** 2026-08-20 14:21:07
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

BUG-463 (`task/bug/verified/463_filters_gpu_texture_framebuffer_leak.md`, High severity, 🎯
Verified) found `examples/minwebgl/filters`'s `Renderer`/`Framebuffer` never freeing replaced GPU
textures/framebuffers -- `WebGlTexture`/`WebGlFramebuffer` are thin JS handle wrappers, and dropping
the Rust-side value never tells the GL driver to free the underlying GPU resource without an
explicit `gl.delete_texture`/`gl.delete_framebuffer` call, which neither type made on any
replacement path before this fix. The fix -- aliasing-safe `gl.delete_texture` calls in
`Renderer::image_texture_set`/`original_texture_set` (routing the two restore methods through
`image_texture_set` instead of direct field assignment), plus a new `impl Drop for Framebuffer`
deleting both the framebuffer and its color-attachment texture -- is already applied and documented
with `Fix(BUG-463)`/`Root cause`/`Pitfall` comments at 5 sites across `renderer.rs`/`framebuffer.rs`.
This task performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md § Core Procedures :
Procedure - Promote Bug to Task` (PROC12) -- to formally register that already-complete,
already-verified fix as a tracked task, closing BUG-463.
Testable: `cd examples/minwebgl/filters && cargo check --target wasm32-unknown-unknown` → exit 0.

## In Scope

- `examples/minwebgl/filters/src/renderer.rs`'s already-applied aliasing-safe texture deletion in
  `image_texture_set`/`original_texture_set`, and the two restore methods routed through
  `image_texture_set` -- verify present; no further edit expected.
- `examples/minwebgl/filters/src/framebuffer.rs`'s already-applied `impl Drop for Framebuffer` --
  verify present; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/463_filters_gpu_texture_framebuffer_leak.md`'s header back to this
  task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `examples/minwebgl/filters` -- the fix is complete and independently
  verified by the bug's own VERIFY Gate.
- Re-running BUG-463's own VERIFY Gate -- already run and recorded in the bug file's Verification
  Record; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- BUG-503 (`task/bug/completed/503_filters_cancel_deletes_the_texture_it_restores.md`, already ✅
  Completed) -- a distinct, separately-filed and separately-closed downstream defect in the same
  aliasing guard this fix introduced (a self-assignment case BUG-463's own guard didn't cover); out
  of scope here, already fully closed under its own ID.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Fix already applied: aliasing-safe `gl.delete_texture` in `Renderer`'s two setters (both restore
  methods routed through `image_texture_set`); `impl Drop for Framebuffer` deleting both the
  framebuffer and its color-attachment texture.
- Green state already confirmed, and re-confirmed live during this task's filing:
  `cargo check -p filters --target wasm32-unknown-unknown` compiles clean.
- No refactor needed -- the fix adds deletion calls and a `Drop` impl, no structural churn beyond
  that.
- Fix documentation already complete at the bug level: BUG-463 carries the full Root Cause/Why Not
  Caught/Fix Location/Prevention narrative in its own body -- this task does not duplicate it, only
  cross-links via `closes: BUG-463`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention -- document rather than force/spoof if
  so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -n "self.gl.delete_texture" examples/minwebgl/filters/src/renderer.rs` | Whole-file scan for the aliasing-safe deletion calls | present in both `image_texture_set` and `original_texture_set` |
| T02 | `grep -n "impl Drop for Framebuffer" examples/minwebgl/filters/src/framebuffer.rs` | Whole-file scan for the `Drop` impl | present, deletes both handle and color_attachment |
| T03 | `grep -n "Fix(BUG-463)" examples/minwebgl/filters/src/renderer.rs examples/minwebgl/filters/src/framebuffer.rs` | Whole-file scan for the fix comment across both files | present at all documented sites |
| T04 | `cd examples/minwebgl/filters && cargo check --target wasm32-unknown-unknown` | crate compiles for wasm32 | 0 errors |

## Acceptance Criteria

- `Renderer::image_texture_set`/`original_texture_set` call `self.gl.delete_texture` on the
  outgoing handle unless it aliases the sibling field
- `Framebuffer` implements `Drop`, deleting both `handle` and `color_attachment`
- The fix's source comments carry all 3 required fields: `Fix(BUG-463)`, `Root cause`, `Pitfall`
- `cargo check -p filters --target wasm32-unknown-unknown` succeeds
- `task/bug/verified/463_filters_gpu_texture_framebuffer_leak.md`'s header states `**Fix Task:**`
  pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify -- an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 -- Does `image_texture_set`/`original_texture_set` delete the outgoing texture unless it
  aliases the sibling field?
- [ ] C2 -- Does `Framebuffer` implement `Drop`, deleting both the framebuffer handle and its
  color-attachment texture?
- [ ] C3 -- Do the fix's source comments carry `Fix(BUG-463)`, `Root cause`, and `Pitfall` fields?
- [ ] C4 -- Does `cargo check -p filters --target wasm32-unknown-unknown` succeed with 0 errors?

**Registration correctness**
- [ ] C5 -- Does this task's `closes:` field name `BUG-463`?
- [ ] C6 -- Does BUG-463's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 -- No Edit/Write tool call in this task's own execution targeted
  `examples/minwebgl/filters/src/renderer.rs` or `src/framebuffer.rs` (the fix content matches
  what BUG-463's own already-completed fix applied; this task made no further source edit to it --
  note this repo's working tree carries many pre-existing, unrelated uncommitted changes from
  other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 -- `grep -c "delete_texture\|delete_framebuffer" examples/minwebgl/filters/src/renderer.rs examples/minwebgl/filters/src/framebuffer.rs`
  → ≥ 3 (two setter deletions in `renderer.rs` plus the `Drop` impl's two deletions in
  `framebuffer.rs`)

### Invariants

- [ ] I1 -- `cargo check -p filters --target wasm32-unknown-unknown` → 0 errors

### Anti-faking checks

- [ ] AF1 -- the aliasing guard genuinely compares the outgoing handle against the sibling field
  before deleting (`self.image_texture.as_ref() == Some( &old )` or equivalent), not an
  unconditional delete that would risk a use-after-delete on the aliased case -- checked by
  reading the guard's condition itself, not just the presence of a `delete_texture` call

## Verification Record

**Gate Round 1** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | In Scope names the two already-applied source changes; Out of Scope excludes BUG-503 (a distinct, separately-closed downstream defect touching the same guard) and re-running BUG-463's own VERIFY Gate. Adversarial pass: re-read `renderer.rs` live and found the aliasing guard has since been *extended* by BUG-503's own fix (`is_self_assign` check layered onto the same conditional) — confirmed this doesn't invalidate In Scope's claim, since BUG-463's own contribution (the `aliases_original` check against `original_texture`, plus the base `delete_texture` call) is still intact and independently identifiable within the combined guard; documented the layering precisely rather than glossing over it. | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (cites BUG-463 path/severity/state), Observable (names the exact fix mechanism), Scoped (registration only), Testable (wasm32 compile command with expected exit 0). Adversarial pass: checked the Testable line's command actually exercises the fixed code paths (not just a no-op compile) — `filters` is a binary-only example crate with no separate lib target, so `cargo check` does type-check the exact `renderer.rs`/`framebuffer.rs` bodies containing the fix; confirmed sufficient. | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: "does this need a tracked task?" — yes, PROC12 requires every promoted bug to close through a linked task, not left as a bare bug-file fix. No speculative work added beyond the registration + live re-verification. | — |
| D4 | Implementation Readiness | — | 🟢 | Delivery Requirements state the fix is already applied; Test Matrix T01-T04 all executable as literal shell commands. Adversarial pass: ran all 4 for real this round (not merely eyeballing the file) — T01/T02/T03 grep hits confirmed at the exact lines cited, T04 (`cargo check -p filters --target wasm32-unknown-unknown` via longrun) returned exit 0 in 45s. | — |
| D5 | Execution Scope | — | 🟢 | Both touched files (`renderer.rs`, `framebuffer.rs`) resolve inside this repository, under `examples/minwebgl/filters/src/`. No path outside the repo referenced. | — |
| D6 | Crate Scope Unity | — | 🟢 | Every deliverable path resolves inside exactly one crate (`filters`). Adversarial pass: checked whether the `obj_viewer`/other-crate cross-references anywhere in this task's own text (there are none in 507 — that concern applies to 509, not this task) would leak scope; confirmed 507's own text stays entirely within `filters`. | — |
| D7 | Crate Locality | — | 🟢 | Fix and task both target the leaf crate that owns the defect (`filters`), not a pushed-up aggregator (`examples/minwebgl/` itself has no shared source this fix touches). | — |
| D8 | Crate Single Responsibility | — | 🟢 | `filters` crate's responsibility ("apply configurable image filters via WebGL, with a lil-gui control panel") stays statable without "and"; this task's registration work doesn't expand that responsibility. | — |

**Live re-verification (this round, not carried over from the bug's own VERIFY Gate):**
- `grep -n "Fix(BUG-463)" src/renderer.rs src/framebuffer.rs` → 5 hits (renderer.rs:53,103,125,149; framebuffer.rs:58), matching Acceptance Criteria.
- `grep -n "self.gl.delete_texture" src/renderer.rs` → 2 hits (lines 97, 117), one per setter.
- `grep -n "impl Drop for Framebuffer" -A3 src/framebuffer.rs` → present at line 70, `fn drop` deletes both `handle`/`color_attachment` per the bug's own Fix Location.
- Read `src/renderer.rs` lines 45-100 directly: confirmed `image_texture_set`'s guard is `let aliases_original = self.original_texture.as_ref() == Some( &old ); ... if !aliases_original && !is_self_assign { self.gl.delete_texture( Some( &old ) ); }` — BUG-463's own `aliases_original` check is present and functioning exactly as the bug file's Fix Location describes; the additional `is_self_assign` term is BUG-503's later, separate contribution to the same guard (out of this task's scope, not a regression of it).
- `longrun .launch -- cargo check -p filters --target wasm32-unknown-unknown` → exit 0, 45s elapsed, no errors or warnings in the build log.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | FILED | Task created via PROC12 (bug_promote) for BUG-463. |
| 2026-08-20 | READINESS_GATE_PASS | 8/8 dimensions 🟢 on live re-verification; task claimed for verification via `tsk .claim_verify 507`. |
| 2026-08-20 | EXECUTED | Fix was already applied prior to this task's filing (BUG-463's own fix); this task's execution is the registration/verification walk itself, confirmed complete. |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-20 14:16:19 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FILED | task created |
| 2026-08-20 14:23:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | same-actor guard expected to block (filed_by == actor); documenting attempt per project convention |
| 2026-08-20 14:20:52 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-20 14:21:07 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
