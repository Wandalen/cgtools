# Delete primitive_generation's dead contours_to_mesh and fix capability-understating doc

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/primitive_generation
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Two low-stakes `primitive_generation` cleanup items bundled together (P3, dead-code/hygiene bucket): (1)
`src/text/ufo.rs`'s `contours_to_mesh` function (lines 382-545, confirmed by direct read this session) is
marked `#[cfg(feature = "font-processing")] #[allow(dead_code)]` and is NOT included in the crate's own
`mod_interface!` export block (lines 755-765, which only exports `load_fonts, Glyph, Font, text_to_mesh,
text_to_countour_mesh`) — confirmed 100% unreachable from outside the crate; delete it, or wire it into
the public API if it turns out to be intended future surface (check git history/commit messages for intent
before deleting outright); (2) separately, the crate's docs understate its actual capabilities relative to
what the code supports — carried forward from the audit triage plan, re-confirm the specific
under-description against current docs before rewriting.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code) tier
  merged with a P5 (doc drift) item for the same crate, Delete-candidate / Fix-in-place bucket.

- **[2026-08-10]** `IMPLEMENTED` — Both Goal items executed, plus the structural defect that task 055's
  adversarial pass explicitly rerouted to this task (`--features text` alone never compiled).

  **Item 1 — dead `contours_to_mesh` deleted (167 lines, `src/text/ufo.rs`).** Intent check per the
  Goal's own instruction, via whitelisted `git log -S "contours_to_mesh"`: the function originated in
  commit `82659817` (2025-06-26, "add text mesh generation...") inside the demo's local
  `geometry_generation/text.rs`, then rode through crate extraction (`12a4a302`) and rename
  (`8386a5f4`) without any commit message ever expressing future-public-surface intent, and without any
  `mod_interface!` revision ever exporting it. Decisive evidence it is extraction residue, not planned
  surface: the sole plausible consumer — the `text_rendering` example, the original demo this code was
  extracted from — carries its own local `fn contours_to_mesh` (`examples/minwebgl/text_rendering/
  src/text.rs:410`) and never imported the crate's copy. Deleted outright; workspace-wide grep confirms
  zero remaining references outside the example's own local implementation. The deletion orphaned 4
  imports (`std::rc::Rc`, `std::cell::RefCell`, `F32x4`, `AttributesData`) — caught by the verification
  battery's own `RUSTDOCFLAGS="-D warnings"` doc-test stage (log `-0020`, exit 101; plain `cargo check`
  had only warned invisibly) and trimmed.

  **Item 2 — doc drift re-confirmed against current docs before rewriting (per the Goal), found in BOTH
  directions, then fixed.** Understated: the readme's two "planned but not yet implemented" placeholder
  blocks sat exactly where real, working capability exists (`curve_to_geometry` ribbon meshing; the
  full UFO text-to-mesh pipeline). Overstated: `Cargo.toml` declared 3 features gating zero code —
  `csg`, `gltf-import`, `random` — pulling 6 dead optional dependencies (`csgrs`, `parry3d`, `gltf`,
  `rand`, `getrandom`, `interpoli`); grep-proven zero `cfg`-gated code, zero source references, zero
  workspace consumers requesting any of them.

  **Changes:**
  1. `src/text/ufo.rs`: real module re-gated `text` → `font-processing` with a 3-field `Fix(TASK-021)`
     source comment — root cause: the module was gated on the feature it is thematically related to
     (`text`) rather than the feature that provides everything it calls
     (`contours_to_fill_geometry` is `font-processing`-only since task 055). This is the fix for 055's
     rerouted finding: `--no-default-features --features text` now compiles for the first time ever.
  2. Silent stub module deleted: the old `#[cfg(not(feature = "text"))]` stub carried always-return-
     None/empty function bodies with signatures that had drifted from the real ones. Replaced with an
     empty `mod private {}` under `not(font-processing)` plus `#[cfg(feature = "font-processing")]`-
     gated `orphan use` exports (055's precedent) — absence is now a loud compile error naming the
     missing symbol, never a silently wrong runtime result.
  3. `Cargo.toml`: feature graph reduced to what exists — `enabled` (core), `text = ["enabled",
     "dep:kurbo"]` (path flattening), `font-processing = ["text", "dep:earcutr", "dep:norad",
     "dep:quick-xml"]` (UFO fonts + triangulation), `full`. The 3 dead features and 6 dead optional
     dependencies removed. `csgrs` deliberately retained as a *workspace* dependency — the
     `narrow_outline` and `text_rendering` examples use it directly — so the `core2` git patch
     (BUG-007/task 008) stays required and untouched.
  4. `readme.md`: honest Features bullets; new "Feature Flags" section listing the 4 real flags with
     self-verification commands (`cargo doc -p primitive_generation --features font-processing --open`;
     `cargo check -p primitive_generation --no-default-features --features text`); both placeholder
     blocks replaced with real, compiling snippets — Curve to Ribbon and Text to 3D — which are now
     executable doc tests, so they can no longer drift silently; honest Dependencies list.

  **Verification** (log `-0022`, exit 0, 371s — all run directly, package-scoped, detached via
  `longrun`): (1) `RUSTFLAGS="-D warnings" cargo check -p primitive_generation` default — clean;
  (2) `--no-default-features --features text` — clean, first-ever pass for this combination;
  (3) `--all-features` — clean; (4) `cargo nextest run --all-features` — 5/5;
  (5) `RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features` — 3/3 (the readme snippets);
  (6) `cargo clippy --all-targets --all-features -- -D warnings` — clean; (7) `cargo check` of all 5
  real consumers (`lottie_surface_rendering`, `animation_surface_rendering`,
  `curve_surface_rendering`, `character_control`, `text_rendering`) — clean.

- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`). Confirming pass re-verified the final state directly: workspace grep for
  `contours_to_mesh` (only the example's own local fn remains), ufo.rs gate inventory (exactly three
  `font-processing` gates: real module, empty stub, export group), fresh `git status` scoped to the
  crate (exactly the 3 intended files modified, no strays), and the full 7-stage battery log. The
  adversarial pass produced two real findings, both resolved in-loop: (a) the Goal's mandated
  git-history intent check had not demonstrably been performed before deletion — closed by running
  `git log -S` (whitelisted read) and recording the extraction-residue evidence trail above; (b) probed
  the feature-deletion blast radius for hypothetical out-of-workspace consumers — the deleted features
  gated zero code, so enabling them was always a no-op; removal converts a silent nothing into a loud
  manifest error, which is strictly better, and zero workspace consumers request them. Also re-probed
  the empty-stub design under default and text-alone builds (battery stages 1–2 green) and confirmed
  the first battery run's stage-5 failure (orphaned imports, log `-0020`) was fixed and re-verified,
  not waved through. All 15 dimensions PASS; state → ✅ Completed.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Both Goal items executed; the `text`→`font-processing` re-gate is the exact defect task 055's Out of Scope explicitly routed here by name | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (dead code misleads; docs drift), Observable (grep/compile), Scoped (one crate, named files), Testable (compile matrix + doc tests) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skip → 167 dead lines keep misleading readers, an advertised feature combination (`text` alone) stays permanently uncompilable, readme keeps placeholder blocks where real features exist, and 6 unused dependencies keep inflating builds | — |
| D4 | Implementation Readiness | — | 🟢 | Goal's two-option fork (delete vs wire-in) resolved by evidence: never exported, `#[allow(dead_code)]`-marked, zero references, sole plausible consumer owns a local duplicate | — |
| D5 | Execution Scope | 🟡 | 🟢 | Adversarial pass caught that the Goal-mandated git-history intent check had not demonstrably run before deletion — closed via `git log -S`: function born in demo code (`82659817`), carried through extraction/rename, no intent statement anywhere | Intent check performed and its evidence trail recorded in `IMPLEMENTED` |
| D6 | Crate Scope Unity | — | 🟢 | All edits confined to `primitive_generation` (`src/text/ufo.rs`, `Cargo.toml`, `readme.md`) | — |
| D7 | Crate Locality | — | 🟢 | Fix targets the exact crate owning the dead code and drifted docs; no aggregator touched; `csgrs` kept at workspace level for the 2 examples that genuinely use it | — |
| D8 | Crate Single Responsibility | — | 🟢 | Responsibility narrowed to what the crate actually does — feature list now matches real exports | — |
| B1 | Rulebook Compliance | — | 🟢 | House codestyle throughout; cfg-gated `orphan use` block mirrors 055's precedent in `primitive.rs`; delete-don't-archive (no stub kept "for compatibility") | — |
| B2 | Test-First Requirement | — | 🟢 | Compile-time defects — the compile matrix is the RED/GREEN signal (`text`-alone E0432 pre-fix → clean post-fix); readme snippets promoted to executable doc tests (3 pass), adding coverage that did not exist before | — |
| B3 | Evidence of Failure | — | 🟢 | RED on record twice: 055's documented E0432 repro at reroute time; this task's own battery stage-5 exit 101 (log `-0020`) on the orphaned imports | — |
| B4 | Proper Fix Only | — | 🟢 | Root-cause gate move (gate on the feature providing what the module calls), not per-call-site cfg sprinkling; loud absence instead of silent stubs; deletion instead of archiving | — |
| B5 | Fix Verification | 🔴 | 🟢 | First battery run failed at stage 5 (4 orphaned imports, log `-0020`, exit 101); imports trimmed; full 7-stage relaunch clean end-to-end (log `-0022`, exit 0): checks under 3 feature sets, nextest 5/5, doc tests 3/3, clippy, 5 consumers | Import trim + full battery relaunch |
| B6 | Knowledge Preservation | — | 🟢 | 3-field `Fix(TASK-021)`/`Root cause`/`Pitfall` comment at the re-gate site; no new test file — compile-time defect with no assertable runtime behavior (055's precedent); readme doc tests are the durable executable documentation | — |
| B7 | Code Cleanliness | — | 🟢 | Fresh `git status --short` scoped to the crate: exactly `Cargo.toml`, `readme.md`, `src/text/ufo.rs` modified; no backup/stray files; no commented-out code retained | — |
| **Total** | | 🔴 | 🟢 | 2 (both resolved in-loop) | 2/2 |

**Aggregate verdict:** PASS — all 15 dimensions clean on the final pass, zero Blocking Findings open.
Two findings surfaced during the loop and were fixed rather than waved through: the missing
Goal-mandated intent check (D5, closed with `git log -S` evidence) and the first battery run's
stage-5 doc-test failure (B5, closed by trimming the orphaned imports and re-running the entire
7-stage battery, not just the failed stage). D1–D8 are the Readiness Verification Gate dimensions;
B1–B7 apply because this task, like 055, fixes a genuine compile-breaking defect alongside its
hygiene scope.
