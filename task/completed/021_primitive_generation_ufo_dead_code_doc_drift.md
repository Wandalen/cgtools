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

## Verification

### Checklist

- [x] C1 — Is `contours_to_mesh` genuinely deleted from `primitive_generation` (not merely unexported)? Workspace-wide `grep -rn "contours_to_mesh" --include="*.rs" .` → `2` hits total, both in `examples/minwebgl/text_rendering/src/text.rs` (that example's own pre-existing local duplicate, per the History's own citation); `0` hits anywhere under `module/helper/primitive_generation`.
- [x] C2 — Is `src/text/ufo.rs`'s real module now gated on `font-processing` (the fix this task made after task 055's adversarial pass rerouted the finding here), not the original `text`? Current source: line 11 `#[ cfg( feature = "font-processing" ) ]` directly above `mod private` (line 12), preceded by a 3-field `Fix(TASK-021)` comment (lines 5-10) citing the exact root cause (module gated on `text` while calling `font-processing`-only `contours_to_fill_geometry`).
- [x] C3 — Is the old silent-stub module (drifted-signature always-`None`/empty fallbacks) genuinely replaced with a loud, empty stub? Current source lines 537-540: `#[ cfg( not( feature = "font-processing" ) ) ] mod private { }` — a genuinely empty module body, no fallback function definitions at all. Under that cfg, referencing any of `load_fonts`/`Glyph`/`Font`/`text_to_mesh`/`text_to_countour_mesh` is now a compile-time "not found" error, never a silently-wrong runtime result.
- [x] C4 — Are exactly the 3 claimed-dead features (`csg`, `gltf-import`, `random`) and 6 claimed-dead dependencies (`csgrs`, `parry3d`, `gltf`, `rand`, `getrandom`, `interpoli`) genuinely absent from `Cargo.toml`? `grep -nE "csg|gltf-import|random|csgrs|parry3d|^gltf|getrandom|interpoli" Cargo.toml` → `0` hits. Current `[features]` block holds exactly 5 keys (`default`, `enabled`, `full`, `text`, `font-processing`); current `[dependencies]` holds 9 crates plus a separate `[dependencies.web-sys]` table (10 total).
- [x] C5 — Do the readme's claimed Feature Flags section and 2 new executable doc-test snippets (Curve to Ribbon Mesh, Text to 3D Geometry) still exist? Confirmed via direct read: `readme.md:33-40` (Feature Flags section, both self-verification commands present verbatim — `cargo doc -p primitive_generation --features font-processing --open` and `cargo check -p primitive_generation --no-default-features --features text`); `readme.md:69-75` and `:81-90` (the two ```rust,no_run``` code blocks).

### Measurements

- [x] M1 — `src/text/ufo.rs` line count: `553` (was: `765` — `git show 4469eafb^:module/helper/primitive_generation/src/text/ufo.rs | wc -l` → `765`), a net `-212` lines. Note, investigated honestly: this is larger than the History's own claimed "167 lines" for the `contours_to_mesh` deletion in isolation — the same implementing commit (`4469eafb`) also bundled the `text`→`font-processing` re-gate and stub replacement (this task's own Goal item 2 follow-on, C2/C3 above) into the same file, so the measured net diff is not attributable to the dead-code deletion alone. Both claimed changes are independently confirmed present; the discrepancy is an imprecise historical citation, not a functional defect.
- [x] M2 — `Cargo.toml` named `[features]` keys: `5` (was: `8` per `git show 4469eafb^:module/helper/primitive_generation/Cargo.toml`), a `-3` delta matching the claimed removal exactly. `[dependencies]` entries (incl. `web-sys`): `10` (was: `16`), a `-6` delta matching the claimed removal exactly.

### Invariants

- [x] I1 — Crate check, default features: `longrun`-launched `cargo check -p primitive_generation` → clean, `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 10.32s`, exit 0.
- [x] I2 — Crate check, `--no-default-features --features text` (the exact combination this task's History claims to have made buildable "for the first time ever"): `longrun`-launched `cargo check -p primitive_generation --no-default-features --features text` → clean, exit 0. Confirmed still holding today.
- [ ] I3 — Crate check, `--all-features` (needed to reach this task's own `font-processing`-gated readme doc test): `longrun`-launched `cargo check -p primitive_generation --all-features` → currently FAILS: `error[E0639]: cannot create non-exhaustive struct using struct expression` at `src/text/ufo.rs:368` and `:83`, exit 101 — same newly-introduced, unrelated regression documented in task 018's Verification (commit `5f33be66`, 2026-08-11, one day after this task's own 2026-08-10 verification; `mingl::geometry::BoundingBox` gained `#[ non_exhaustive ]`, breaking two pre-existing struct-literal sites in `ufo.rs` this task never touched). Consequently `cargo test -p primitive_generation --doc --all-features` (the readme's 3 doc tests, including this task's own 2 new snippets) also cannot currently compile, exit 101.
- [ ] I4 — Lint cleanliness, default features: `longrun`-launched `cargo clippy -p primitive_generation --all-targets -- -D warnings` → currently FAILS, but not on this task's code: `error: could not compile browser_log (lib) due to 1 previous error` — same `5f33be66`-introduced, unrelated `#[ allow( clippy::exhaustive_structs ) ]`-without-`reason` violation documented in tasks 018/055's Verification sections (`module/helper/browser_log/src/panic.rs:82`, workspace `Cargo.toml:117`'s `allow_attributes_without_reason = "warn"` lint). `primitive_generation`'s own code contributes zero clippy findings.

### Anti-faking checks

- [x] AF1 — Guards against `contours_to_mesh` silently reappearing (e.g. re-copied from the example's local duplicate): re-run C1's workspace grep — must still show `0` hits inside `module/helper/primitive_generation`.
- [x] AF2 — Guards against the feature graph silently regrowing dead entries: re-run C4's grep for the 3 removed features / 6 removed dependencies — must still return `0` hits in `Cargo.toml`.
- [x] AF3 — Guards against trusting this task's original "7-stage battery, all clean" History claim without re-running it: I3/I4 above are direct proof that a fully-passing verification can go stale from an unrelated commit landing the very next day. Before citing this task's readme doc tests, `--all-features` build, or clippy run as currently working — e.g. as evidence the crate is healthy — re-run `cargo check -p primitive_generation --all-features`, `cargo test -p primitive_generation --doc --all-features`, and `cargo clippy -p primitive_generation --all-targets --all-features -- -D warnings` fresh; do not assume a prior day's PASS still holds.

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
