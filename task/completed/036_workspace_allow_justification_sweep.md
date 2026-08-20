# Workspace-wide sweep: justify or remove unexplained #[allow] attributes

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/helper/primitive_generation/src/text/ufo.rs` has 8 blanket `#![allow(clippy::...)]` attributes
(lines 4-11, confirmed by direct read this session) with zero justification comments — used as the
concrete first-hand example of a systemic pattern the audit found repeated across the workspace (P8 —
mechanical hygiene tier). Sweep every `#[allow(...)]`/`#![allow(...)]` attribute workspace-wide
(`grep -rn "#!\?\[allow("`); for each, either add a one-line comment explaining the specific reason the
lint is suppressed, or remove the attribute and fix the underlying lint if it's not actually justified.
**This is a large, mechanical, cross-cutting sweep — likely worth decomposing per-crate at pickup** rather
than one giant diff, similar to task 035's own decomposition note.

## In Scope

- `module/helper/primitive_generation/src/text/ufo.rs`: swept all 8 blanket `#![allow(clippy::...)]` —
  removed 3 stale, fixed 4 in code (`explicit_iter_loop`, `uninlined_format_args`, `needless_continue`,
  `semicolon_if_nothing_returned`), re-justified 1 as a scoped `#[allow(clippy::too_many_lines)]` on
  `from_glif`
- Workspace-wide census of `#[allow(...)]`/`#![allow(...)]` sites (1905 across 102 crates) and the
  lint-inheritance map, to size and inform the remainder of the sweep

## Out of Scope

- The remaining ~1897 `#[allow]` sites across the other 101 crates — decomposed into successor
  `draft/058` (per-crate procedure), not fixed by this task
- Unrelated pre-existing regressions surfaced during verification (`browser_log/panic.rs` missing a
  lint-reason string; `mingl::BoundingBox` becoming `#[non_exhaustive]` and breaking `ufo.rs`'s own
  struct-literal construction) — confirmed unrelated to this task's edits, left unfixed

## Verification

### Checklist

- [x] C1 — Does `ufo.rs` currently carry exactly the one claimed-justified `#[allow(...)]` attribute (scoped, not blanket) and zero of the 8 original blanket `#![allow(clippy::...)]` attributes? `grep -n "allow(" module/helper/primitive_generation/src/text/ufo.rs` → exactly 1 hit: line 120, `#[ allow( clippy::too_many_lines ) ]` (item-scoped, no `#!` file-level prefix).
- [x] C2 — Does the pre-fix baseline actually match the claimed 8-attribute, 4-lint-fixed/3-stale-removed/1-rejustified disposition? `git show 0046f840:module/helper/primitive_generation/src/text/ufo.rs | grep -n "allow("` → exactly 8 blanket `#![allow(clippy::...)]` at lines 4-11 (`needless_continue`, `cloned_instead_of_copied`, `explicit_iter_loop`, `unnecessary_cast`, `too_many_lines`, `semicolon_if_nothing_returned`, `uninlined_format_args`, `redundant_closure_for_method_calls`) plus one unrelated pre-existing `#[allow(dead_code)]` at line 381 — matches the Goal's "lines 4-11" claim and the History's exact lint-name disposition (3 stale removed, 4 fixed in code, 1 re-justified) exactly.
- [x] C3 — Does the successor task (`draft/058`) this task decomposed its remainder into actually exist and remain live? `task/draft/058_workspace_allow_sweep_per_crate.md` exists; `health.md`'s own "Open work streams" section confirms it is actively being incremented (renderer 87→42, tiles_tools 460→38 allow counts).
- [x] C4 — Does `primitive_generation`'s own scoped build/test (independent of the workspace-wide clippy invocation blocked per I3/I4 below) still succeed? `cargo check -p primitive_generation --lib` (default features) → exit 0, 20.83s; `cargo nextest run -p primitive_generation` (default features) → 3/3 passed.

### Measurements

- [x] M1 — Live `allow(` attribute count in `ufo.rs`: `1` (was: `8` blanket `#![allow(clippy::...)]` at lines 4-11 pre-fix — confirmed via `git show 0046f840:module/helper/primitive_generation/src/text/ufo.rs`, the last commit before this task's own fix landed; commit `4469eafb` already shows the fixed `1`-attribute state).

### Invariants

- [x] I1 — Default-feature build (unaffected by the drift in I4): `cargo check -p primitive_generation --lib` → exit 0, `Finished` in 20.83s.
- [x] I2 — Default-feature test suite: `cargo nextest run -p primitive_generation` → `3 tests run: 3 passed, 0 skipped`, exit 0.
- [x] I3 — DRIFT (blocked, unrelated to this task's scope): `cargo clippy -p primitive_generation --all-targets --all-features -- -D warnings` (the exact command this task's History claims produced zero warnings) → **currently fails**, exit 101, reproduced twice. Root cause is `#[ allow( clippy::exhaustive_structs ) ]` without a reason string at `module/helper/browser_log/src/panic.rs:82` (`clippy::allow_attributes_without_reason`, implied by `-D warnings`) — a file this task never touched, pulled in transitively (`primitive_generation` → `renderer`/`minwebgl` → ... → `browser_log`). Re-running with `--keep-going` confirms the block is structural (nothing downstream of `browser_log` in this chain can be reached), not a transient fluke.
- [x] I4 — DRIFT (genuine regression, discovered this session, unrelated to this task's own edits): `cargo check -p primitive_generation --all-targets --all-features` (bypasses the clippy-only lint in I3) → **also currently fails**, exit 101, with 2 real `E0639` "cannot create non-exhaustive struct" errors directly inside `ufo.rs` itself, at lines 83 and 368 (`let bounding_box = BoundingBox { min: ..., max: ... }` / `max_size: BoundingBox { min, max }`). Root cause: `module/min/mingl/src/geometry.rs:14` added `#[ non_exhaustive ]` to `BoundingBox` (committed in `HEAD`, confirmed via `git show HEAD:module/min/mingl/src/geometry.rs`; `mingl` now provides `BoundingBox::new(min, max)` as the replacement constructor) in a later, unrelated refactor that never updated these two call sites. Confirmed reproducible (2 clean runs); confirmed NOT scoped to `--all-features` alone — `cargo check --workspace` (default features) hits the same error, because several example crates' `Cargo.toml` (`lottie_surface_rendering`, `character_control`, `animation_surface_rendering`, `curve_surface_rendering`) request `primitive_generation`'s `text`/`font-processing` feature, which workspace-wide feature unification then applies even to a "default-features" workspace build.

### Anti-faking checks

- [x] AF1 — Guards against the fixed attributes silently regressing back to a blanket file-level suppression: `grep -n "^#!\[" module/helper/primitive_generation/src/text/ufo.rs` must keep returning 0 hits; the sole surviving `#[allow(clippy::too_many_lines)]` must remain item-scoped to `from_glif`, never promoted back to file scope.
- [x] AF2 — Guards against conflating "this task's fix broke" with "an unrelated dependency broke": I3/I4 are real, currently-reproducible failures, but neither touches the code this task changed (allow-attribute lines 4-11 vs. the unrelated `BoundingBox` construction at lines 83/368, and the unrelated `browser_log` file). A future re-check must re-run C1/C2/M1 (which need no successful compilation) to confirm this task's own fix is undisturbed independently of whether I3/I4's unrelated regressions have since been fixed.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P8 (mechanical
  hygiene) tier, Fix-in-place bucket (cross-cutting).
- **[2026-08-10]** `IMPLEMENTED` — Executed per the Goal's own decomposition note: sized the sweep
  (census: **1905 allow sites across 102 crates** — not one diff), built the decision-critical
  inheritance map (72/102 crates have `[lints] workspace = true`; non-inheriting: mdmath_core,
  ndarray_cg, embroidery_tools + 27 example crates, whose blanket allows largely suppress lints that
  are not even enabled for them), and discovered `[workspace.lints.clippy]` already centrally allows
  several of the commonly file-suppressed lints WITH justification comments (Cargo.toml 71-98) — so
  many file-level copies are pure redundancy, and example crates share a copy-pasted template block
  reducible to one template-level decision. Executed the Goal's named concrete instance empirically:
  removed all 8 blanket `#![allow(clippy::...)]` from `primitive_generation/src/text/ufo.rs`, ran
  scoped clippy — only 5 of 8 lints actually fire (3 were stale suppressions: cloned_instead_of_copied,
  unnecessary_cast, redundant_closure_for_method_calls — stayed removed). Fixed the code for 4:
  explicit_iter_loop ×4 (`&mut` loop forms), uninlined_format_args ×2, needless_continue
  (`_ => continue` → `_ => {}`, behavior-identical as last loop statement), semicolon_if_nothing_returned.
  Re-added 1 as a scoped fn-level attribute with justification: too_many_lines on `from_glif` (117-line
  linear XML event state machine). Verified: `cargo clippy -p primitive_generation --all-targets
  --all-features` now ZERO warnings, `cargo test -p primitive_generation --all-features` all pass
  (2+3 unit + 3 doc), both exit 0. Remainder decomposed into draft/058 (per-crate procedure proven here,
  census table, inheritance map, examples-template tranche).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) the confirming plan assumed all 8 ufo.rs suppressions were live — empirical
  removal proved 3 stale, changing the fix from "justify 8" to "remove 3, fix 4, justify 1"; (2) the
  seemingly-equivalent `_ => continue` → `_ => {}` rewrite was verified against the surrounding control
  flow (match is the loop body's final statement) before applying — in any other position it would have
  changed behavior; (3) initial per-line-number fix targeting was abandoned after line drift between
  clippy passes — switched to content-anchored edits.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's decomposition note followed: census + inheritance map + concrete instance + successor draft | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | One successor draft, not 25; examples tranche reduced to a single template decision | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Successor carries the proven 5-step per-crate procedure | — |
| D5 | Execution Scope | 🟢 | 🟢 | Code edits confined to ufo.rs; draft/058 + index the only other writes | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Proper-fix-over-suppression honored: 4 fixed in code, only 1 justified suppression retained | — |
| B2 | Test-First | 🟡 | 🟢 | Plan assumed all 8 suppressions live; empirical clippy run disproved 3 | Stale trio removed outright instead of justified |
| B3 | Evidence of Failure | 🟢 | 🟢 | Pre-fix clippy log: 9 warnings (5 unique lints) on record in -0002_longrun.log | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | `_ => continue` → `_ => {}` is behavior-identical ONLY as the loop's final statement | Control-flow position verified before edit |
| B5 | Fix Verification | 🟢 | 🟢 | Post-fix: clippy ZERO warnings, tests 2+3+3 pass, both exit 0 | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Census, inheritance map, and procedure live in draft/058; justification comment lives on from_glif | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No line-number-targeted edits survived (drift caught); content-anchored edits only | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved | 2/2 |
