# 509: Register object_picking meshes_load material pairing fix closes BUG-465

## Execution State

- **id:** 509
- **title:** Register object_picking meshes_load material pairing fix closes BUG-465
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-20 14:16:37
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/object_picking
- **closes:** BUG-465
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-20 14:21:07
- **expires_at:** 2026-08-20 16:21:07
- **unverified_at:** 2026-08-20 14:20:52
- **unverified_by:** unknown
- **verifying_at:** 2026-08-20 14:21:07
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## MOST Goal

BUG-465 (`task/bug/verified/465_object_picking_meshes_load_material_paired_by_position.md`, Medium
severity, 🎯 Verified) found `examples/minwebgl/object_picking`'s `meshes_load` pairing models with
materials via `.zip( materials )` -- positional pairing that silently applies the wrong material
whenever a model's `material_id` doesn't match its own position, and silently drops trailing models
whenever `materials.len() < models.len()` (the shorter iterator wins). The fix -- replacing the
`.zip` with an explicit `model.mesh.material_id.and_then( | id | materials.get( id ) )` lookup,
matching the reference pattern already used correctly in the sibling `obj_viewer` example -- is
already applied and documented with a `Fix(BUG-465)`/`Root cause`/`Pitfall` comment at the fix site
in `src/main.rs`. This task performs the remaining lifecycle bookkeeping -- `tsk.rulebook.md §
Core Procedures : Procedure - Promote Bug to Task` (PROC12) -- to formally register that
already-complete, already-verified fix as a tracked task, closing BUG-465.
Testable: `cd examples/minwebgl/object_picking && cargo check --target wasm32-unknown-unknown` →
exit 0.

## In Scope

- `examples/minwebgl/object_picking/src/main.rs`'s already-applied `material_id`-based lookup in
  `meshes_load`, replacing the positional `.zip( materials )` pairing -- verify present; no
  further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/465_object_picking_meshes_load_material_paired_by_position.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `examples/minwebgl/object_picking` -- the fix is complete and
  independently verified by the bug's own VERIFY Gate.
- Re-running BUG-465's own VERIFY Gate -- already run and recorded in the bug file's Verification
  Record; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- Modifying the sibling `obj_viewer` example's own `GLMesh::from_tobj_model` -- that is the
  reference pattern this fix was aligned to, already correct, untouched by this fix.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Fix already applied: `meshes_load`'s `for ( model, material ) in models.iter().zip( materials )`
  replaced with `for model in models` plus
  `let material = model.mesh.material_id.and_then( | id | materials.get( id ) );`; the
  texture-loading branch updated to `material.and_then( | m | m.diffuse_texture.as_ref() )`.
- Green state already confirmed, and re-confirmed live during this task's filing:
  `cargo check -p object_picking --target wasm32-unknown-unknown` compiles clean.
- No refactor needed -- the fix is scoped to `meshes_load`'s pairing loop and its immediate
  texture-lookup consumer.
- Fix documentation already complete at the bug level: BUG-465 carries the full Root Cause/Why Not
  Caught/Fix Location/Prevention narrative in its own body, including the hand-traced 3-model/
  2-material hypothetical -- this task does not duplicate it, only cross-links via
  `closes: BUG-465`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention -- document rather than force/spoof if
  so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -n "Fix(BUG-465)" examples/minwebgl/object_picking/src/main.rs` | Whole-file scan for the fix comment | present at the `meshes_load` fix site |
| T02 | `grep -n "material_id.and_then" examples/minwebgl/object_picking/src/main.rs` | Locate the replacement lookup | present, keyed by `model.mesh.material_id` |
| T03 | `grep -n "\.zip( materials )\|.zip(materials)" examples/minwebgl/object_picking/src/main.rs` | Whole-file scan for the removed positional pairing | absent (0 matches) |
| T04 | `cd examples/minwebgl/object_picking && cargo check --target wasm32-unknown-unknown` | crate compiles for wasm32 | 0 errors |

## Acceptance Criteria

- `meshes_load` pairs each model with its material via `model.mesh.material_id`, not iteration
  position
- A model with no material or an out-of-range `material_id` degrades to `None` (no texture)
  instead of being silently dropped from the returned `Vec<Mesh>`
- The fix's source comment carries all 3 required fields: `Fix(BUG-465)`, `Root cause`, `Pitfall`
- `cargo check -p object_picking --target wasm32-unknown-unknown` succeeds
- `task/bug/verified/465_object_picking_meshes_load_material_paired_by_position.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify -- an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 -- Does `meshes_load` pair each model with its material via `model.mesh.material_id`
  rather than positional `.zip`?
- [ ] C2 -- Does a model with `material_id = None` or an out-of-range id degrade to no texture
  rather than being dropped from the output?
- [ ] C3 -- Does the fix's source comment carry `Fix(BUG-465)`, `Root cause`, and `Pitfall`
  fields?
- [ ] C4 -- Does `cargo check -p object_picking --target wasm32-unknown-unknown` succeed with 0
  errors?

**Registration correctness**
- [ ] C5 -- Does this task's `closes:` field name `BUG-465`?
- [ ] C6 -- Does BUG-465's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 -- No Edit/Write tool call in this task's own execution targeted
  `examples/minwebgl/obj_viewer/src/mesh.rs` (the reference pattern this fix aligned to; already
  correct, untouched -- note this repo's working tree carries many pre-existing, unrelated
  uncommitted changes from other concurrent activity, so a blanket repo-wide `git diff --stat` is
  not a meaningful signal here).

### Measurements

- [ ] M1 -- `grep -c "material_id" examples/minwebgl/object_picking/src/main.rs` → ≥ 2 (the
  lookup itself plus its use in the texture-loading branch)

### Invariants

- [ ] I1 -- `cargo check -p object_picking --target wasm32-unknown-unknown` → 0 errors

### Anti-faking checks

- [ ] AF1 -- the replacement lookup genuinely reads `model.mesh.material_id` (the correct
  cross-reference field per `tobj::Model`'s own shape) rather than re-deriving a positional index
  under a different name that would silently reproduce the original defect -- checked by reading
  the lookup expression itself, not just confirming `.zip` is gone

## Verification Record

**Gate Round 1** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | In Scope names the already-applied lookup replacement; Out of Scope excludes modifying `obj_viewer`'s own already-correct reference pattern. Adversarial pass: T03's Test Matrix row literally claims `.zip( materials )` is "absent (0 matches)" from the file — live grep actually returned 1 match, at line 401, but reading it in context shows it's inside the `// Root cause:` comment *quoting* the old, now-removed code, not executable code. Documenting this precisely rather than letting the literal "0 matches" claim stand uncorrected: the executable `.zip(materials)` pairing is genuinely gone; the comment's mention of the old pattern for explanatory purposes is expected and correct, matching this repo's established `Fix(BUG-NNN)`/`Root cause`/`Pitfall` documentation convention. | T03 note added to Verification (below) clarifying the comment-vs-code distinction. |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (cites BUG-465 path/severity/state), Observable (names the exact lookup mechanism replacing `.zip`), Scoped (registration only), Testable (wasm32 compile command). Adversarial pass: confirmed the Testable line's compile check does exercise `meshes_load`'s body (not dead code) — it's called from `main`'s async load path, included in every build of this binary-only crate. | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: "does this need a tracked task?" — yes, same PROC12 requirement as the other 5 bugs this round. No speculative extension into `obj_viewer` or other crates using similar patterns. | — |
| D4 | Implementation Readiness | — | 🟢 | Delivery Requirements state the fix is already applied; Test Matrix T01-T04 all executable. Adversarial pass: ran all 4 for real this round — T01/T02 grep hits confirmed, T03 required the comment-vs-code correction noted under D1, T04 (`cargo check -p object_picking --target wasm32-unknown-unknown` via longrun) returned exit 0 in 13s. | — |
| D5 | Execution Scope | — | 🟢 | Touched file (`main.rs`) resolves inside this repository, under `examples/minwebgl/object_picking/src/`. | — |
| D6 | Crate Scope Unity | — | 🟢 | Every deliverable path resolves inside exactly one crate (`object_picking`). The `obj_viewer` cross-reference in this task's own text is read-only context (citing the reference pattern the fix was aligned to), not a deliverable path — confirmed no edit or verification command in this task targets `obj_viewer`'s own files. | — |
| D7 | Crate Locality | — | 🟢 | Fix and task both target the leaf crate that owns the defect (`object_picking`), not pushed up to a shared `examples/minwebgl/` helper. | — |
| D8 | Crate Single Responsibility | — | 🟢 | `object_picking` crate's responsibility stays statable without "and"; this task's registration work doesn't expand it. | — |

**Live re-verification (this round, not carried over from the bug's own VERIFY Gate):**
- `grep -n "Fix(BUG-465)" src/main.rs` → present at line 399, full Root cause comment through line 403+.
- `grep -n "material_id.and_then\|material_id" src/main.rs` → confirmed the replacement lookup at line 416: `let material = model.mesh.material_id.and_then( | id | materials.get( id ) );`.
- `grep -n "\.zip( materials )\|.zip(materials)" src/main.rs` → 1 match at line 401 — verified by reading context that this is the `// Root cause:` comment's own quotation of the removed pre-fix code (`` `models.iter().zip( materials )` assumes... ``), not a surviving code path; the executable pairing logic no longer uses `.zip`.
- `longrun .launch -- cargo check -p object_picking --target wasm32-unknown-unknown` → exit 0, 13s elapsed, no errors or warnings.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | FILED | Task created via PROC12 (bug_promote) for BUG-465. |
| 2026-08-20 | READINESS_GATE_PASS | 8/8 dimensions 🟢 on live re-verification (1 genuine adversarial catch: T03's "0 matches" claim corrected to "0 matches in executable code; 1 match inside an explanatory comment quoting the old code"); task claimed for verification via `tsk .claim_verify 509`. |
| 2026-08-20 | EXECUTED | Fix was already applied prior to this task's filing (BUG-465's own fix); this task's execution is the registration/verification walk itself, confirmed complete. |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-20 14:16:37 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | FILED | task created |
| 2026-08-20 14:23:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | same-actor guard expected to block (filed_by == actor); documenting attempt per project convention |
| 2026-08-20 14:20:52 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-20 14:21:07 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
