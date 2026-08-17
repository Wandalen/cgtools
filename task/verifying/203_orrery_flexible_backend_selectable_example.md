# Implement `orrery_flexible`'s 4-Backend-Selectable Orrery Scene Renderer

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-16 19:39:31
- **expires_at:** 2026-08-16 21:39:31
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/orrery/flexible
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **repo_identity:** self
- **in_motion:** true
- **verifying_at:** 2026-08-16 19:39:31
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

Replace `examples/orrery/flexible/src/main.rs`'s current reserved stub (a `println!`
and a `compile_error!` feature guard) with a real implementation that loads the
shared orrery `scene.rhai` document (per the family's Scene contract —
`examples/orrery/readme.md`) and renders it through whichever of the 4
`gpu_hal`-routed backends is selected via Cargo feature
(`webgl`/`webgpu`/`wgpu`/`vulkan`), plus trunk/`index.html` wiring for the two
browser features. Motivated by this being the actual product deliverable the whole
ADR-004 chain exists to unblock — the orrery family's stated goal of comparing the
same visual scene across every backend, now including a single feature-selectable
crate rather than four separate ones. Not blocked on task 202: the `webgl`, `webgpu`,
and `wgpu` feature paths route through `gpu_hal` backends that already exist and
work today (confirmed compiling/clippy-clean as of this task's filing), so those 3 backends can
be implemented and verified immediately; only the `vulkan` feature path's
implementation and tests depend on task 202 (`gpu_hal`'s real `vulkan` backend)
landing first — see `## In Scope`'s per-feature breakdown. Testable: the native
build (`wgpu`/`vulkan` features) renders the orrery scene to an offscreen target and
exits 0; `trunk build` succeeds for the `webgl`/`webgpu` features and the resulting
page renders real, non-blank pixels when checked via `browsee`, mirroring tasks
191/197/198's own browsee pixel-verification precedent.

## In Scope

- Real `main.rs` per selected feature: scene loading via `scene_script`'s existing
  Rhai-document loader (same pattern `webgpu/src/main.rs` and `webgpu/src/scene.rs`
  already use), `gpu_hal::Device` construction routed by feature (`webgl`→WebGL
  constructor, `webgpu`→WebGPU constructor, `wgpu`→`Device::new_native`,
  `vulkan`→`Device::new_vulkan` from task 202), render loop drawing the scene's
  sun/orbit-ring/planet/nebula/star elements through `gpu_hal`'s resource API
- `index.html` (new) for the two browser features, mirroring
  `examples/gpu_hal/triangle_browser`'s and `examples/orrery/webgpu`'s existing
  trunk wiring
- Real call site for the `mingl` scene-loading glue already declared (but unused) in
  `Cargo.toml`

**Per-feature dependency note:** the `webgl`, `webgpu`, and `wgpu` feature paths
above route through `gpu_hal` backends that already exist and work today — that
work can start and land independently of task 202. Only the `vulkan` feature path
(`Device::new_vulkan` call site) depends on task 202's `gpu_hal` vulkan backend
having landed first; if task 202 is not yet done, implement and verify the other 3
features first and defer only the `vulkan`-feature slice (main.rs's `vulkan` arm,
T02, T-vulkan-specific Acceptance Criteria) until it is.

## Out of Scope

- Real multi-pass bloom / advanced per-backend visual effects — the `webgl`
  planned-member bullet in `examples/orrery/readme.md` separately calls out "real
  multi-pass bloom" as that standalone member's differentiator; `flexible/`'s bar is
  matching the existing `webgpu` reference implementation's fidelity, not exceeding it
- Promoting `scene.rhai` out of `webgpu/scene/` into the shared family directory —
  `examples/orrery/readme.md`'s own Family conventions section already names this as
  a deferred step ("When a second member lands, it is promoted to this directory");
  a natural follow-up, but its own cross-member file-move decision, not bundled
  silently into this task
- Any further `gpu_hal`/`minvulkan` API changes — those are tasks 201/202's own
  scope; this task only calls the APIs they establish (task 201 already landed;
  task 202 only for the `vulkan` feature path — see the Per-feature dependency
  note under `## In Scope`)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not
by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements (no bloom,
    no scene-file relocation — see Out of Scope)
-   `cargo nextest run -p orrery_flexible --features wgpu` and
    `--features vulkan` pass with zero failures and zero warnings
    (`RUSTFLAGS="-D warnings" cargo clippy -p orrery_flexible --all-targets --features <each> -- -D warnings`
    exits 0 for all 4 features)
-   `trunk build` succeeds for `--features webgl` and `--features webgpu`
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Build and run with `--features wgpu` (default) | native | Loads the scene, renders to an offscreen target, exits 0 |
| T02 | Build and run with `--features vulkan` | native | Same as T01, via the vulkan backend |
| T03 | `trunk build` with `--features webgl` | wasm32 | Build succeeds, produces `dist/` artifacts |
| T04 | `trunk build` with `--features webgpu` | wasm32 | Build succeeds, produces `dist/` artifacts |
| T05 | `browsee` pixel check against the built webgl page | browser | Non-blank real pixels at a known scene landmark (e.g. sun disc center), mirroring tasks 191/197/198's own browsee pixel-verification style |
| T06 | `browsee` pixel check against the built webgpu page | browser | Same as T05, via the webgpu backend |
| T07 | Build with zero backend features (`--no-default-features`) | any | `compile_error!` still fires (existing feature-guard behavior preserved) |

## Acceptance Criteria

-   `main.rs` loads the shared orrery scene and renders it for all 4 features —
    no feature falls back to the reserved stub's `println!`
-   `index.html` exists and trunk-builds cleanly for `webgl`/`webgpu`
-   Every row T01–T07 in `## Test Matrix` has a corresponding passing test or
    documented manual verification step (browsee runs)
-   At least one pixel-level assertion per browser backend (T05/T06) — not merely
    that the page loads without a JS console error
-   `readme.md`'s Status line no longer reads "reserved — this is a non-functional skeleton"

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT
self-verify — an independent verifier performs the walk after the task reaches
🔎 Accepting.

### Checklist

**Scene fidelity**
- [ ] C1 — Does every feature load the same `scene.rhai` document (no per-backend
      scene forks)?
- [ ] C2 — Does the render loop draw all scene element families named in the family
      readme (sun, orbit rings, planets/moons, nebula, star field, HUD grid) — or, if
      any are deferred, is that deferral explicit in this task's own Out of Scope
      rather than silently dropped?

**Backend purity (ADR-004 conformance)**
- [ ] C3 — Does `cargo tree -p orrery_flexible --features webgl` (and `webgpu`,
      `vulkan`) show no `wgpu` dependency, direct or transitive? (`wgpu` should
      appear only under `--features wgpu`)

**Browser wiring**
- [ ] C4 — Does `index.html` follow the same trunk-wiring shape as
      `examples/gpu_hal/triangle_browser`/`examples/orrery/webgpu`?

**Documentation**
- [ ] C4b — Does `readme.md`'s Status line no longer read "reserved — this is a
      non-functional skeleton"?

**Pixel proof**
- [ ] C5 — Do the T05/T06 browsee checks assert on specific pixel content (not
      merely "page did not crash")?

**Out-of-Scope confirmation** (`## Out of Scope` absence checks)
- [ ] C6 — Confirms NO multi-pass bloom or advanced per-backend visual effects were
      added beyond matching the existing `webgpu` reference implementation's fidelity
      (`git diff -- examples/orrery/webgpu/` → empty; effects live only in `flexible/`
      if at all, and only at reference-implementation parity)
- [ ] C7 — Confirms `scene.rhai` was NOT relocated out of `webgpu/scene/`
      (`git status examples/orrery/webgpu/scene/scene.rhai` → no rename/move; `flexible/`
      loads it via relative reference, not a local copy)
- [ ] C8 — Confirms zero diff to `module/helper/gpu_hal/` and `module/min/minvulkan/`
      (`git diff --stat -- module/helper/gpu_hal/ module/min/minvulkan/` → empty —
      this task only calls their existing APIs)

### Measurements

- [ ] M1 — `main.rs` line count per feature path: `wc -l src/main.rs` (was: 21 lines,
      reserved stub)
- [ ] M2 — `cargo tree -p orrery_flexible --features webgl | grep -c '^wgpu'` → `0`
      (confirms browser features stay wgpu-free)

### Invariants

- [ ] I1 — `cargo nextest run -p orrery_flexible --features wgpu` → 0 failures
- [ ] I2 — `cargo nextest run -p orrery_flexible --features vulkan` → 0 failures
- [ ] I3 — `RUSTFLAGS="-D warnings" cargo clippy -p orrery_flexible --all-targets --features <each of 4> -- -D warnings` → 0 warnings, all 4 features
- [ ] I4 — `trunk build` → exit 0, for both `webgl` and `webgpu`

### Anti-faking checks

- [ ] AF1 — T05/T06's browsee pixel assertions check specific, non-background pixel
      values at a known scene landmark, not merely "canvas is not entirely one
      solid color" (which a partially-broken render could still satisfy)
- [ ] AF2 — T01/T02's "renders to an offscreen target" claim is backed by an actual
      pixel/frame assertion, not just a process-exit-0 check (a native build that
      constructs a device and renders nothing would otherwise still pass a bare
      exit-code check)

## Related Documentation

- `docs/adr/004_native_vulkan_hal_backend.md` — governs this crate's whole 4-backend
  feature-to-backend mapping
- `examples/orrery/readme.md` — family conventions, Scene contract, `flexible/`'s
  Directory table entry this task fulfills
- `examples/orrery/flexible/readme.md` — this crate's own readme, Status line
  updated by this task
- `examples/orrery/webgpu/src/main.rs`, `examples/orrery/webgpu/src/scene.rs` — the
  reference implementation this task's scene-loading path mirrors
- `docs/layer/002_l1_gpu_hal.md` — L1 status card, browser-pixel-verification
  precedent (tasks 191/197/198) this task's own T05/T06 follow
- `task/executed/202_gpu_hal_vulkan_backend.md` — dependency scoped to this
  task's `vulkan` feature path only (not a blanket block — see the Per-feature
  dependency note under `## In Scope`); provides the `Device::new_vulkan` API that
  feature path consumes (implementation complete and passing — 📦 Executed,
  pending acceptance verification)

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 19:39:31 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_VERIFY_PASS | `tsk .verify_pass 203` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard, consistent with task 206 precedent; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk`, following user-directed
  orrery backend expansion (ADR-004). Goal: real 4-backend orrery rendering, the
  product deliverable tasks 201/202 unblock. Not blocked overall; only the
  `vulkan` feature path depends on task 202 (see In Scope's per-feature note).
- **[2026-08-16]** `VERIFIED` — Readiness Verification Gate passed (Tier 2, 8/8 🟢)
  after a Fix-and-Recheck Loop: Round 1 found a blanket `blocked_by: 202` defect
  (D2/D4) and an Out-of-Scope Checklist-coverage gap (caught during `/doc_tsk`
  Step 7's Task Quality Gate); both fixed in place and re-verified. Moved to `task/`.
- **[2026-08-16]** `AMENDED` — Round 1 domain pass (Task Quality Gate · TA122)
  found and fixed 2 further gaps: missing `repo_identity` field and missing
  AC↔Checklist mapping for the readme Status-line update. No Readiness Gate
  verdict changed.
- **[2026-08-16]** `NORMALIZED` — Task Normalization (PROC7) found the file
  sitting at `task/` root while marked 🎯 Verified — a State-Location Mismatch
  (`tsk.rulebook.md § Normalization : State-Location Mismatch Anti-patterns ·
  TA067`, Anti-pattern 5). Per that anti-pattern's fix and
  `§ Task Lifecycle : Procedure - VERIFY Transition`'s precondition, the prior
  Verification Record was not trusted at face value: state reset to
  ❓ (Unverified), file moved to `task/unverified/`, `tsk .claim_verify`
  re-claimed it (🔬 Verifying), and the full 8-dimension Readiness Verification
  Gate was re-run from scratch. Re-run PASSED 8/8, no dimension regressions;
  fixed 2 minor staleness issues found along the way — the `## Related
  Documentation` cross-reference to task 202 (moved `task/` → `task/executed/`
  since original filing) and a session-relative phrase in `## Goal`. Reasserted
  🎯 (Verified) via `tsk .verify_pass`; moved to `task/verified/`.
- **[2026-08-16]** `CLAIM_VERIFY` — task re-claimed for verification
  (`verifying_at`/`verifying_by` populated, `round: 1`, state reset to
  🔬 Verifying); this transition itself was not historized at the time it
  occurred, backfilled now for consistency. Fresh 8-dimension Readiness
  Verification Gate re-run PASSED 8/8 (see `## Verification Record` below).
  `tsk .verify_pass` refused with the same-actor sandbox guard
  (`self-verification forbidden (actor matches filed_by)` — see `## Journal`);
  not force/spoofed — task remains at 🔬 Verifying pending a different
  verifying actor.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name                      | Prev | Now | Issues                                                                                                   | Fixes                                                                                                          |
| ---- | ------------------------- | ---- | --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| D1   | Scope Coherence           | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D2   | MOST Goal Quality         | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D3   | Value / YAGNI             | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D4   | Implementation Readiness  | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D5   | Execution Scope           | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D6   | Crate Scope Unity         | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D7   | Crate Locality            | —    | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |
| D8   | Crate Single Responsibility | —  | 🟢   | —                                                                                                                                                              | —                                                                                                                                                                                            |

**Pass 1 (Confirming):** D1 — In/Out of Scope remain non-empty and mutually
exclusive; the deliverable (feature-routed `main.rs` + `index.html` in one crate)
is a meaningful observable outcome, not just "write a file". D2 — Goal is
motivated (unblocks the ADR-004 product deliverable), observable (names `main.rs`,
`index.html`, the 4 feature paths), scoped (one crate, bounded deliverable), and
testable (T01–T07 name concrete commands/checks). D3 — Null Hypothesis: skipping
this task leaves `orrery_flexible` a non-functional stub and the entire ADR-004
chain (tasks 201/202) without the product deliverable it exists to unblock —
concrete committed need, no speculative scope. D4 — Delivery Requirements name
concrete commands (`cargo nextest`, `cargo clippy`, `trunk build` per feature);
Test Matrix (T01–T07) and Acceptance Criteria are specific and traceable to
Checklist items C1–C8. D5/D6 — every deliverable path (`examples/orrery/flexible/`)
and every boundary-check path (C3/C6–C8) resolves inside this repository and inside
exactly this one crate. D7 — `examples/orrery/flexible` is the correct leaf owner
of "load scene, render via 4 selectable backends" glue; not orchestration-only. D8
— crate responsibility remains statable in one sentence without "and" (render the
shared scene through 4 selectable backends).

**Pass 2 (Adversarial):** Attempted to find a reason each dimension does NOT hold,
independent of the prior (untrusted) record. D1 — checked whether the Per-feature
dependency note creates an implicit 4-way scope split that should be decomposed
into separate tasks; no — one crate with feature-gated code paths is one
deliverable under D6/D7/D8's own single-crate-single-task posture, a delayed
sub-slice within one task is not a scope-split signal. D2 — found the Goal
paragraph's parenthetical "(confirmed compiling/clippy-clean this session)" uses
session-relative phrasing that goes stale the moment it's read in a later session;
this doesn't break Testable (the stated success command is unaffected) but is a
genuine drive-by defect — fixed to "as of this task's filing". D3 — checked whether
task 202's current lifecycle position (📦 Executed, not yet ✅ — blocked on the
same-sandbox acceptance guard) invalidates the Null Hypothesis; no — the `vulkan`
feature's real prerequisite is working `gpu_hal` code, which is confirmed landed
and passing, independent of 202's own tsk-lifecycle position. D4 — re-verified
C3/C6–C8's `cargo tree`/`git diff`/`git status` commands target paths that exist
in the repo today; all executable as written. Independently re-checked
`## Related Documentation` (not assumed unchanged from the untrusted prior pass)
and found the task-202 cross-reference stale — it still said "now 🎯 Verified"
though 202 has since moved to `task/executed/`; fixed to cite the current path and
state. No further defects found on this pass; verdict matches the (untrusted)
prior record's, now independently reconfirmed rather than assumed.

**Amendment history (from the pre-normalization record, preserved for
traceability):** the original Readiness Verification Gate (2026-08-16) passed
8/8 after a Fix-and-Recheck Loop that fixed a blanket `blocked_by: 202` defect
(D2/D4) and an Out-of-Scope Checklist-coverage gap; a subsequent Round 1 domain
pass (Task Quality Gate · TA122) fixed 2 further gaps (missing `repo_identity`,
missing AC↔Checklist mapping for C4b). See `## History` for the full chronology.
