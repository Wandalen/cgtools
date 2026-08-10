# Decide vectorizer's fate: fix feature-gate blocker and re-enable, or delete

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/vectorizer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/helper/vectorizer` is commented out of root `Cargo.toml`'s workspace `members` list
(`# "module/helper/vectorizer", # TODO: Fix feature gate issues`), and the workspace-dependencies entry
for it points at the wrong path (`path = "module/vectorizer"` instead of the real
`module/helper/vectorizer`) — both confirmed by direct read this session, and both newly-discovered
findings not present in the original audit. The crate has 14 source files but zero test files and zero
cross-references from any other crate (confirmed via workspace-wide grep this session). This is a
decision task (P3 bucket): investigate what "feature gate issues" actually blocks compilation, then
either (a) fix the feature-gate problem, correct the dependency path, and re-add to `members`, with tests
added since none exist today, or (b) if the crate is genuinely unmaintained/superseded, delete it
entirely (Delete-candidate). Whichever direction, fix the wrong dependency path as part of the same
change.

**Related Tasks:** `056` (`task/draft/056_vectorizer_revival_watch_item.md`) — a placeholder task filed
to keep the door open on this task's DELETE decision for a future revival if a real consumer emerges.
Does not reopen or otherwise change this task's own terminal state; see this task's History entry below.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (decision point)
  tier, Delete-candidate/Fix-in-place decision bucket.

- **[2026-08-10]** `DECIDED_AND_EXECUTED` — Investigated what "feature gate issues" actually blocks by
  temporarily re-adding the crate to workspace `members` (with the dependency path corrected to
  `module/helper/vectorizer`) and running `cargo check -p vectorizer` across every individual feature
  (`default`, `cli`, `serialization` alone, `random` alone, `--all-features`). Found the problem is
  materially larger than a missing `#[cfg]` attribute:

  1. `src/actions/error.rs` unconditionally applies `serde::Serialize`/`serde_with::serde_as` derive and
     helper attributes to the `Error` enum, even though the backing `use serde;`/`use serde_with::..`
     imports are correctly gated behind `#[cfg(feature = "serialization")]` — the attributes themselves
     were never gated to match.
  2. `src/actions/layers.rs` and `src/actions/clusters.rs` (the `actions` layer, unconditional core
     functionality per `actions.rs`'s own doc comments — "Layers vectorization method",
     "Clusters vectorization method") unconditionally `use commands::raster::vectorize::{layers,clusters}`
     to import their `Config`/`ColorDifference`/`CLIArgs`/`Hierarchical` types — but the entire `commands`
     layer is `#[cfg(feature = "cli")]`-gated (`src/commands.rs`). This is a genuine architectural
     inversion, not a missing gate: the CLI-only layer's `clap::Parser`-derived config structs are the
     *only* config types that exist for the core vectorization algorithms — `actions::layers::action()`
     takes `commands::raster::vectorize::layers::CLIArgs` (a clap-specific struct bundling `InputOutput` +
     `Config`) directly as its argument. The "library" (`enabled`-only, no `cli`) use case has apparently
     never actually compiled since this split was introduced — confirmed by `default`-features `cargo
     check` failing with the identical `commands::raster` E0433 errors, not just the serde ones.
  3. `src/actions/common.rs` unconditionally `use fastrand::Rng` (gated behind the separate `random`
     feature) and unconditionally references `commands::InputOutput` (same `cli`-only-layer problem as
     #2).

  A correct fix requires relocating the config types out of the CLI layer into `actions` (or a shared
  location) and having `commands` depend on `actions` instead of the reverse, for both `layers` and
  `clusters` — real cross-module type relocation, not a `#[cfg]` patch — plus writing a test suite from
  scratch (zero tests exist today; task's own Fix-in-place path requires adding them, and this crate
  processes raster images, so meaningful tests need real fixture images and output assertions, not just
  unit stubs).

  **Decision: DELETE.** Weighed against fix-in-place:
  - Zero cross-references confirmed by direct repo-wide grep this session (`grep -rln vectorizer
    --include=*.toml --include=*.rs --include=*.md .` outside the crate itself: only `Cargo.toml`
    (workspace registration), `locales.md` (generated crate catalog), and this task's own files — no
    other crate imports or depends on it).
  - Zero tests, so a fix-in-place has no regression safety net for a raster-processing algorithm during
    the very refactor that would relocate its config types.
  - The actual defect is a genuine architectural inversion (core depends on CLI-only types), not the
    "TODO: Fix feature gate issues" one-line-fix the comment implied — proportionate effort is well
    beyond a P3 backlog item, with zero current consumer to justify it now (YAGNI).
  - Deletion is not data loss — full history recoverable via `git show` on the pre-deletion tree if a
    concrete need for raster-to-vector conversion arises later, at which point a real consumer can drive
    the correct core/CLI boundary design instead of a speculative one.

  **Executed:** removed `module/helper/vectorizer/` entirely (`rm -rf`); removed its commented-out
  `members` entry and the (path-wrong) `[workspace.dependencies.vectorizer]` block from root `Cargo.toml`
  (both entries deleted outright, not left commented). `locales.md` still lists the crate — left untouched
  since the file is generator-maintained (`Do not edit manually... Maintained by .locale.doc.generate`)
  and has no static per-crate config entry in `locales.config.yml` to correct; it will self-correct on
  the next regeneration pass, out of this task's scope. Verified via `cargo metadata --no-deps` (clean
  resolve) and a full `longrun`-launched `cargo check --workspace --all-features` (exit 0, 10s, zero
  errors) that removal introduces no breakage anywhere else in the workspace.

- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`). Confirming pass re-read the executed diff (`git status --short`: ~19 files
  under the deleted crate directory plus root `Cargo.toml` modified) against the History entry above and
  found it accurate. Adversarial pass re-ran the cross-reference search with the file-type restriction
  removed (`grep -rl "vectorizer" . --exclude-dir=.git --exclude-dir=target` vs. the original
  `--include=*.toml --include=*.rs --include=*.md`) and found 2 real hits the narrower search had missed:
  `action/run` (a stale comment illustrating "real product, not example" crates by name) and
  `script/test_workspace.sh` (an active step running `cargo check -p vectorizer ...`, which would now fail
  since the package no longer exists — confirmed `test_workspace.sh` is referenced only from
  `script/readme.md`'s responsibility table, not from any CI/automation, but the step was still a live,
  guaranteed-broken command). Both fixed in place: removed the vectorizer check block from
  `test_workspace.sh` (comment + echo + if/else/fi, matching Delete-Don't-Archive — no commenting out);
  replaced `action/run`'s stale crate name with `embroidery_tools`, a real, currently-existing
  `module/helper` crate that serves the same illustrative purpose. Also independently re-checked the
  `locales.md` self-correction claim rather than resting on it: confirmed neither of the file's own
  2 stated sources of truth (`locales.config.yml`, `.persistent/locale.toml`) exist anywhere in this
  repo (both `find` searches empty), so self-correction is expected-by-design (the generator almost
  certainly rebuilds from a live directory scan) but not independently verifiable from within this repo —
  recorded as such rather than overclaiming confirmation. Post-fix re-verification: repo-wide grep now
  returns only `locales.md` (accepted staleness, reasoned above) and this task's own file/index row —
  zero unaccounted references remain. All 8 dimensions PASS after the loop; state → ✅ Completed.

- **[2026-08-10]** `RELATED_TASK_LINKED` — User asked to "keep [the vectorizer question] open for now"
  after reviewing this task's DELETE decision and execution, without reverting the already-executed,
  already-verified code deletion. Checked `tsk.rulebook.md` and confirmed `✅ Completed` is a strict
  terminal state in the current rulebook version (v5.13) — the REOPEN transition was removed entirely
  (`§ Vocabulary : Regression Event`: "create a new task and link it via `**Related Tasks:**` to the
  original... REOPEN transition is removed"; T2 state machine shows no outgoing transition from `✅`).
  Moving this file back to `task/draft/` would have violated that invariant. Instead filed task `056`
  (`task/draft/056_vectorizer_revival_watch_item.md`) as a Deduplication Search Case E match (closed task,
  differing scope) and added a bidirectional `**Related Tasks:**` cross-link. This task's own state,
  location, and all prior content remain unchanged — this entry and the `**Related Tasks:**` line above
  are the only edits made.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Task uses the lighter Draft-stage Goal-only format (no separate In Scope/Out of Scope sections); the Goal paragraph itself names the exact unit and the bounded fix-or-delete decision space | — |
| D2 | MOST Goal Quality | — | 🟢 | Draft-stage Goal didn't name an explicit verification command up front; execution supplied rigorous verification anyway (per-feature-combination `cargo check` pre-decision, full-workspace `cargo check` post-deletion) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skipped → crate stays commented-out/unreachable forever. Delete chosen over speculative fix-in-place for a zero-consumer, zero-test crate — the more YAGNI-aligned outcome | — |
| D4 | Implementation Readiness | — | 🟢 | No formal Test Matrix — doesn't map onto a deletion decision; compile-clean-before/after used as the equivalent evidence | — |
| D5 | Execution Scope | 🔴 | 🟢 | Initial cross-reference grep (`--include=*.toml,*.rs,*.md`) missed 2 real hits: `action/run`'s illustrative comment and `script/test_workspace.sh`'s now-broken `-p vectorizer` check step; broadened unrestricted grep found both. `locales.md` still lists the deleted crate (left untouched, generator-maintained) — confirmed neither of its 2 stated sources of truth exist anywhere in this repo, so self-correction is expected-by-design but not independently verifiable from here | Removed the broken vectorizer check block from `script/test_workspace.sh`; replaced `action/run`'s stale example crate name with `embroidery_tools` (a real, current `module/helper` crate) |
| D6 | Crate Scope Unity | — | 🟢 | Root `Cargo.toml` touch is the one legitimate exception to single-crate scope — deleting a crate necessarily updates the one registry tracking workspace membership | — |
| D7 | Crate Locality | — | 🟢 | Pure deletion; nothing added to any aggregator or wrong-leaf crate | — |
| D8 | Crate Single Responsibility | — | 🟢 | N/A (crate deleted, not modified) — vacuously holds | — |
| **Total** | | 🔴 | 🟢 | 2 (resolved) | 2/2 |

**Aggregate verdict:** PASS — one Blocking Finding on D5 (2 missed cross-references found by broadening the adversarial-pass grep beyond the original file-type-restricted search), fixed in place via a self-contained Fix-and-Recheck Loop, re-verified by a follow-up unrestricted grep returning zero unaccounted hits. All other 7 dimensions clean on both the confirming and adversarial pass. D1–D8 are the Readiness Verification Gate dimensions (`tsk.rulebook.md § Task File : Readiness Verification Gate`), reused at completion per this session's established precedent for investigate-and-resolve tasks (e.g. task 011); this is a decision-and-delete task rather than a classic code bug fix, so the separate Bug-Fixing Task Quality Requirements (B1–B7) do not apply.
