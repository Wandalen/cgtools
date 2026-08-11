# Fix primitive_generation's missing font-processing feature gate on earcutr usage

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

`primitive_generation`'s `Cargo.toml` declares `earcutr` as an optional dependency, only pulled in by the
`font-processing` feature (`font-processing = ["text", "dep:earcutr"]`). But
`contours_to_fill_geometry` (`src/primitive.rs`) calls `earcutr::earcut(..)` unconditionally, with no
`#[cfg(feature = "font-processing")]` gate around either the call site or the function itself. Building
the crate with default features only — `cargo check -p primitive_generation`, no `--all-features` —
fails with `error[E0433]: cannot find module or crate 'earcutr' in this scope` at the unconditional call
site. Discovered as a byproduct of task 018's investigation (not part of either of that task's two named
issues — no doc contradiction, no NaN) and independently re-confirmed directly:
`cargo check -p primitive_generation` (default features) → E0433 at the exact cited call site.

**Not currently caught by any verification gate:** every command this workspace's test/CI machinery
runs (`will .test l::3`, `cargo nextest`, `cargo clippy`, etc.) passes `--all-features`, so this break is
invisible to the standard verification loop — it only manifests for a consumer building with default
features alone, e.g. `cargo build -p primitive_generation` or as a default-feature dependency from
another crate.

**Resolution is a design decision, not a mechanical fix — two candidate directions, pick one at
pickup:**
1. Gate `contours_to_fill_geometry` (and any other `earcutr`-using code path) behind
   `#[cfg(feature = "font-processing")]`, matching how `path_to_points` is already gated behind
   `#[cfg(feature = "text")]` in the same `mod_interface!` block — but this changes
   `contours_to_fill_geometry`'s public API surface (no longer callable without the feature), so check
   all current callers first.
2. Make `earcutr` a non-optional dependency (drop it from `font-processing`'s feature list, remove
   `optional = true`) if triangulation-via-`earcutr` is actually a core, always-needed capability rather
   than a text/font-specific one — re-examine why it was gated behind `font-processing` in the first
   place before choosing this path.

## Out of Scope

- The two TASK-018 issues themselves (doc-contradicting silent failure on triangulation `Err`;
  NaN-producing precondition gap in `curve_to_geometry`) — already fixed and closed separately.
- `text/ufo.rs` dead-code/doc-drift cleanup (task 021).
- The `csgrs`/`core2` yanked-dependency issue in this same crate (BUG-007/task 008).

## History

- **[2026-08-10]** `FILED` — Discovered as a byproduct of task 018's fix (silent failure + NaN gap in
  `primitive_generation`); independently re-confirmed via direct `cargo check -p primitive_generation`
  (default features) → E0433 unresolved-crate error at the unconditional `earcutr::earcut(..)` call
  site in `contours_to_fill_geometry`. Filed separately per this workspace's out-of-scope discipline —
  distinct from task 018's two named issues, task 021, and BUG-007/task 008, all sharing this crate.

- **[2026-08-10]** `IMPLEMENTED` — Chose resolution direction 1 (gate behind `font-processing`, matching
  `path_to_points`'s existing `#[cfg(feature = "text")]` precedent) over direction 2 (making `earcutr`
  non-optional): triangulation-via-`earcutr` is exclusively used by font/text geometry generation, not a
  core always-needed capability, so narrowing its public API surface is the architecturally correct
  choice — confirmed safe by checking every current caller first (`grep` across all consumer
  `Cargo.toml` files): the 4 real external consumers requesting this crate
  (`examples/minwebgl/lottie_surface_rendering`, `animation_surface_rendering`,
  `curve_surface_rendering` — each `features = ["font-processing"]` — and `character_control` —
  `features = ["full"]`, which includes `font-processing`) already request the feature, so none are
  broken by the narrower surface.

  **Changes made** in `module/helper/primitive_generation/src/primitive.rs`:
  1. Added `#[ cfg( feature = "font-processing" ) ]` directly above `pub fn contours_to_fill_geometry`,
     plus a 3-field `Fix(TASK-055)`/`Root cause`/`Pitfall` source comment explaining the optional-
     dependency gating pitfall (a `use`/`dep:` line being correctly gated is not sufficient if the call
     site itself isn't).
  2. Split the `mod_interface!` export block: moved `contours_to_fill_geometry` out of the ungated
     `orphan use { curve_to_geometry, contours_to_fill_geometry, plane_to_geometry };` group into its own
     `#[cfg(feature = "font-processing")] orphan use { contours_to_fill_geometry };` block, mirroring
     `path_to_points`'s existing `#[cfg(feature = "text")]` block immediately below it.
  3. Split the combined `use gl::{ F32x2, F32x4, geometry::BoundingBox };` import: `BoundingBox` is
     `contours_to_fill_geometry`'s only consumer in this file, so gating the function without also
     gating its now-orphaned import produced a fresh `unused import` warning under default features;
     moved `BoundingBox` into its own `#[cfg(feature = "font-processing")] use gl::geometry::BoundingBox;`
     line (mirroring the existing `#[cfg(feature = "text")] use kurbo::PathEl;` pattern), leaving
     `F32x2`/`F32x4` (used elsewhere in the file) ungated.

  In `module/helper/primitive_generation/Cargo.toml`: added a `[[test]] name =
  "contours_to_fill_geometry_test" required-features = ["font-processing"]` entry (with an explanatory
  comment citing TASK-055) — without it, `contours_to_fill_geometry_test.rs` (task 018's existing test
  file, which unconditionally imports the now-gated symbol) would hard-fail to *compile* under default
  features instead of being cleanly skipped.

  **No new unit test was written** — the defect is a compile-time feature-gating gap, not a runtime
  logic error; there is no assertable runtime behavior to test (a missing `#[cfg]` either compiles or
  doesn't). The RED/GREEN signal is the compile command itself: pre-fix, `cargo check -p
  primitive_generation` (default features) failed with E0433; post-fix, it passes. The `required-features`
  Cargo.toml addition provides the equivalent empirical proof for the test-binary boundary: default
  features correctly *exclude* the gated test binary (not fail to build it), while font-processing/
  all-features correctly *include and pass* it — verified directly, not assumed.

  **Verification** — all run directly via Bash, package-scoped, no subagent delegation:
  - `cargo check -p primitive_generation` (default features): clean except one pre-existing, unrelated
    warning (`function contours_to_mesh is never used`, `text/ufo.rs:726`, inside the `#[cfg(not(feature
    = "text"))]` stub module) — analyzed and confirmed this warning is *not* newly created by this fix;
    it was previously masked by the fatal E0433 that killed the whole crate before dead-code analysis
    could run. Squarely task 021's named scope (`text/ufo.rs` dead-code/doc-drift cleanup), already
    excluded by this task's own Out of Scope section. Disappears entirely under `--all-features`
    (confirmed) since the stub module isn't compiled once `text` is on.
  - `cargo check -p primitive_generation --features font-processing`: zero warnings.
  - `cargo check -p primitive_generation --all-features`: zero warnings.
  - `cargo nextest run -p primitive_generation` (default features): 3 tests across 2 binaries — the
    gated test binary is cleanly excluded (not failed).
  - `cargo nextest run -p primitive_generation --all-features`: 5 tests across 3 binaries, all pass,
    including both `contours_to_fill_geometry`-gated tests.
  - `cargo clippy -p primitive_generation --all-targets --all-features -- -D warnings`: exit 0, zero
    warnings (confirmed via a clean log file plus an explicit `grep -iE "warning|error"` sweep returning
    no hits — a first truncated tool-output view had left this ambiguous, so it was re-run and logged to
    get an unambiguous verdict).
  - All 4 real external consumers (`lottie_surface_rendering`, `animation_surface_rendering`,
    `curve_surface_rendering`, `character_control`) independently `cargo check`ed clean.

  **Adversarial finding, investigated and resolved (not a regression):** `cargo check -p
  primitive_generation --no-default-features --features text` (i.e. `text` *without*
  `font-processing`) still fails post-fix — now with E0432 ("`contours_to_fill_geometry`... configured
  out") at `text/ufo.rs`'s `Font` constructor call site, which lives inside `ufo.rs`'s real (non-stub)
  `mod private`, gated only on `#[cfg(feature = "text")]`. Traced via `git show` of the commit that
  captured this fix (`2be3d2cc`, the user's own commit — swept up this task's already-applied working-
  tree changes alongside an unrelated vectorizer-removal commit) to confirm this is **not a new
  regression**: pre-fix, `contours_to_fill_geometry` sat in the *same ungated* `orphan use` group as
  `curve_to_geometry`/`plane_to_geometry`, so its unconditional internal `earcutr::earcut(..)` call was
  already reachable — and already failing (E0433, `earcutr` unavailable) — under *any* feature
  combination lacking `font-processing`, `text`-alone included. This fix only changed the failure's
  shape (E0433 deep inside the function body → E0432 at the caller boundary); `text`-alone was never a
  working configuration, before or after. Confirmed zero real consumers exercise `text` without
  `font-processing` (same 4-consumer grep as above). Disposition: pre-existing defect in `ufo.rs`'s own
  feature-gate design (its `text`-gated code structurally assumes `font-processing`-level functionality),
  squarely `text/ufo.rs` dead-code/doc-drift territory — already named in this task's own Out of Scope
  section as task 021's responsibility, not touched here.

- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`). Confirming pass re-verified the chosen resolution (gate, not
  non-optional-dependency) against the Goal's two candidate directions, re-ran every verification
  command listed above directly rather than trusting the prior summarized state, and re-confirmed via
  fresh `git status`/`git diff` exactly which edits remain uncommitted (only the `BoundingBox` import
  split — the function-level gate, `mod_interface` export split, and Cargo.toml `required-features`
  entry were already captured by the user's own commit `2be3d2cc`). Adversarial pass specifically
  targeted the fix's blast radius across untested feature combinations rather than re-checking what the
  confirming pass already covered: tried `--features text` alone (a combination never explicitly tested
  before this check), found it still fails post-fix, and — instead of accepting that at face value —
  traced the pre-fix commit content to prove it's a pre-existing, non-regressing, zero-consumer,
  out-of-scope defect (see `IMPLEMENTED` entry above for the full trace). All 15 dimensions PASS after
  the adversarial finding was resolved to a Non-Blocking disposition; state → ✅ Completed.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Draft-stage Goal-only format; Goal names the exact unit, the exact defect, and a bounded 2-option resolution space | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (E0433 under default features), Observable (`cargo check` pass/fail), Scoped (one function + its export + one test binary), Testable (explicit verification commands) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skip → any default-features build of this crate, or any dependent crate not requesting `font-processing`, hard-fails to compile — a real, currently-manifesting break, not speculative | — |
| D4 | Implementation Readiness | — | 🟢 | Both candidate resolutions were concrete; direction 1 chosen after checking all 4 real callers first, per the Goal's own explicit instruction to do so | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass found `--features text` alone still fails post-fix (E0432 at `ufo.rs`'s call site) — traced via `git show` of the pre-fix commit content and confirmed pre-existing (same root cause, same combination already broken with E0433 before this fix), zero real consumers, and squarely `text/ufo.rs`/task 021 territory already named in this task's own Out of Scope. Non-Blocking: no fix required, disposition documented | — |
| D6 | Crate Scope Unity | — | 🟢 | All edits confined to `primitive_generation` (`src/primitive.rs`, `Cargo.toml`) | — |
| D7 | Crate Locality | — | 🟢 | Fix targets the exact function/crate owning the `earcutr` dependency; no aggregator or wrong-leaf crate touched | — |
| D8 | Crate Single Responsibility | — | 🟢 | No responsibility change — correcting a feature-gate boundary, not altering what the crate does | — |
| B1 | Rulebook Compliance | — | 🟢 | Gate placement and style exactly mirrors the existing `path_to_points`/`#[cfg(feature = "text")]` precedent in the same file (indentation, brace style, `mod_interface!` block shape) | — |
| B2 | Test-First Requirement | — | 🟢 | No new unit test — the defect is a compile-time feature-gating gap with no assertable runtime behavior. The compile command itself is the RED/GREEN signal (`cargo check` E0433 pre-fix → clean post-fix); `required-features` addition independently verified the test-binary boundary empirically (excluded under default, included+passing under font-processing) | — |
| B3 | Evidence of Failure | — | 🟢 | RED reconfirmed via direct `cargo check -p primitive_generation` (default features) → E0433 at the exact cited call site, both at filing time and independently re-confirmed before implementing | — |
| B4 | Proper Fix Only | — | 🟢 | Gate matches existing codebase precedent exactly (not a novel workaround); verified against actual current callers (not assumed) that narrowing the public API breaks nothing live; addresses the root cause (unscoped optional-dependency usage), not a symptom patch | — |
| B5 | Fix Verification | — | 🟢 | Independently re-ran directly: `cargo check` clean under default (1 unrelated pre-existing warning, analyzed and excluded)/font-processing (0 warnings)/all-features (0 warnings); `cargo nextest` 3/3 default (binary correctly excluded) and 5/5 all-features; `cargo clippy --all-targets --all-features -D warnings` exit 0 confirmed via log file + explicit grep sweep (first truncated view was not trusted); all 4 real external consumers `cargo check` clean | — |
| B6 | Knowledge Preservation | — | 🟢 | 3-field `Fix(TASK-055)`/`Root cause`/`Pitfall` source comment confirmed present via `git show`; Cargo.toml `[[test]]` entry carries its own explanatory comment citing TASK-055; no new test file needed (existing `contours_to_fill_geometry_test.rs` already carries task 018's 5-section doc comments) | — |
| B7 | Code Cleanliness | — | 🟢 | Fresh `git status --short` scoped to the crate shows only `src/primitive.rs` modified (the remaining uncommitted `BoundingBox` import-split); no stray/backup files; the function gate, export split, and Cargo.toml entry were already captured cleanly in the user's own commit `2be3d2cc` | — |
| **Total** | | 🔴 | 🟢 | 1 (resolved, non-blocking) | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both passes, zero Blocking Findings. One adversarial finding (D5: `text`-alone feature combination still fails to compile) was investigated to its root cause via direct `git show` evidence rather than accepted or dismissed at face value, and confirmed to be pre-existing, non-regressing, and already covered by this task's own Out of Scope section — a Non-Blocking Finding per `governance/maav.rulebook.md § MAAV : Severity-Tiered Convergence`, requiring documentation but not a fix. D1–D8 are the Readiness Verification Gate dimensions, B1–B7 are the Bug-Fixing Task Quality Requirements (`tsk.rulebook.md § Bug Fixes : Bug-Fixing Task Quality Requirements`) — both apply here (unlike tasks 023/034) because this is a genuine code bug fix, matching task 018's own 15-dimension precedent in this same crate.
