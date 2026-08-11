# Rewrite tiles_tools/roadmap.md to resolve self-contradiction

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/helper/tiles_tools/roadmap.md` (562 lines, read in full this session) is severely
self-contradictory: it declares Phase 2 "🎉 COMPLETED! 159 passing tests, 2,000+ lines" (lines 66-93) AND
"0/4 milestones complete, Milestone 08 starting today" (lines 131-146) — the closing "Updated Project
Status" block (lines 557-561) agrees with the incomplete version, and no timestamps establish which is
current. P4 (rewrite bucket) — determine actual current implementation state by reading the crate's real
source/tests (not the roadmap's own claims, which are the thing being disputed), then rewrite the roadmap
from that ground truth. Do not merely delete the contradictory sections — the milestone structure itself
may still be useful; the fix is accuracy, not removal.

## Verification

### Checklist

- [ ] C1 — Is `roadmap.md` still internally self-consistent regarding ECS-movement/Flow-Field gap status (the exact defect class this task was filed to fix)? PARTIALLY REGRESSED — `docs/pitfall/002` is genuinely deleted and ECS movement genuinely fixed (task 063; confirmed `ls docs/pitfall/` → only `001`, `003`, `004`, `readme.md`), and roadmap.md's "Known Gaps" table (lines 77-85) and the Phase 3 detail paragraph (line 208) correctly reflect this — but 4 other locations still describe ECS movement/`pitfall/002` as an open gap: the top `Status:` line (5, "Phase 3 substantially complete (1 known gap)"), Quick Start (19, "two documented functional gaps (Flow Fields, ECS movement resolution)"), Ready-to-Code item 3 (24, "[Flow Fields, ECS movement] both already have a documented pitfall"), and Next Priority Actions item 2 (249, "Close `docs/pitfall/002`"). `git show 5f33be66 --stat -- .../roadmap.md` and `git show cd98503d --stat -- .../roadmap.md` both show zero hits for this file — the later architecture refactor never touched it, so the staleness predates it and traces to task 063's own edit not being propagated to every mention it claimed to update.
- [x] C2 — Does `roadmap.md` still correctly reflect that Flow Fields remain genuinely stubbed (the other open gap this task documented)? `grep -c "stub\|Stub" src/flowfield.rs` → `11` hits — pitfall/001 is still live/accurate; this claim has not drifted.
- [x] C3 — Is the `layout.rs` dashboard row (added by this task's own adversarial pass) still present? `grep -n "layout.rs" roadmap.md` → line 59, `Rectangular Layout over Hex Grids | ✅ Complete | layout.rs | ⚠️ no docs/ instance yet`.
- [x] C4 — Are the 2 dead links this task fixed still fixed (`docs/ecs_decision.md` → `docs/architectural_evaluation/001_ecs_library_selection.md`; `src/coordinates/mod.rs` → `src/coordinates.rs`)? `docs/architectural_evaluation/001_ecs_library_selection.md` exists; `docs/ecs_decision.md` and `src/coordinates/mod.rs` both resolve to "No such file or directory"; `roadmap.md:44` cites `src/coordinates.rs`.
- [x] C5 — Does the reproducible-test-count callout (line 75) still instruct re-running the command instead of trusting a static figure? YES, unchanged — "Current, reproducible test count: 237 tests passing (...) Re-run the command yourself for the live number — do not trust a number written into this file, it will drift." The live number has since moved to 245 (M2), exactly as the callout warned it would.

### Measurements

- [x] M1 — `roadmap.md` line count: `303` (was: `561`, `git show 77cc9b9a:module/helper/tiles_tools/roadmap.md | wc -l` — the commit immediately preceding this task's rewrite in the path-filtered log; this task's own Goal text cites "562", a 1-line discrepancy from `wc -l`'s newline-counting convention). This task's rewrite landed at `304` lines (`git show 25ceae76:...roadmap.md | wc -l`); now `303` — 1 line removed later, consistent with task 063's pitfall/002 reference update — net stable.
- [x] M2 — Live `cargo nextest run -p tiles_tools --all-features` count: `245` passed (was: `237`, this task's own investigation figure, also recorded at roadmap.md:75). The +8 is accounted for by task 078 (this task's own documented follow-up) reviving 5 previously-dead flowfield integration tests, plus further reorganization from task 072 — not drift in this task's own work.

### Invariants

- [x] I1 — Test suite (crate-scoped, `--all-features`): `cargo nextest run -p tiles_tools --all-features` → exit 0, 245/245 passed (log `-0009_longrun.log`).
- [ ] I2 — Compiler/lints (crate-scoped, `--all-features`): `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` → exit 101 (log `-0015_longrun.log`). FAILS, but root cause is 3 dependency-hops outside this crate: `browser_log/src/panic.rs:82` (`#[allow(clippy::exhaustive_structs)]` missing a `reason=`, violating the workspace's own `allow_attributes_without_reason = "warn"` under `-D warnings`) via `tiles_tools → animation → mingl/minwebgl → browser_log` (confirmed via `cargo tree -p tiles_tools --all-features -i browser_log`). `browser_log` is unmodified relative to HEAD (`git diff` empty); a concurrent, unrelated workspace-wide sweep is mid-flight converting bare `#[allow(...)]` comments into `reason=`-carrying `#[expect(...)]`/`#[allow(..., reason=...)]` elsewhere in this crate (e.g. `ecs/world.rs`, `ecs/systems.rs`, both currently 1-line-uncommitted for exactly this) but has not yet reached `browser_log`. Not attributable to this task's roadmap.md edit.

### Anti-faking checks

- [x] AF1 — Guards against roadmap.md silently re-drifting into full self-contradiction the way it did before this task: re-running C1's location sweep (`grep -n "pitfall/002\|movement resolution is a no-op" roadmap.md`) after any future edit to `docs/pitfall/`, `ecs/world.rs`, or `roadmap.md` itself must show either zero hits or hits confined to explicitly-dated historical narrative (`Revision History`/`Revision note`) — a hit inside the Status line, Quick Start, or Next Priority Actions sections is the same defect class this task fixed.
- [x] AF2 — Guards against the reproducible-test-count callout (line 75) being replaced with a hardcoded number that goes stale again: any future edit removing the "do not trust a number written into this file" sentence while leaving a bare digit is a regression to the pre-fix pattern.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P4 (doc rewrite)
  tier, Rewrite-from-scratch bucket.
- **[2026-08-10]** `IMPLEMENTED` — Rewrote `module/helper/tiles_tools/roadmap.md` (562 → 304 lines) from
  ground truth instead of the roadmap's own disputed claims. Investigation, in order: (1) `find`-listed
  every `src/**/*.rs` file — discovered `ecs/`, `flowfield.rs`, `field_of_view.rs`, `spatial.rs`,
  `events.rs`, `serialization.rs`, `debug.rs`, `game_systems.rs`, `layout.rs` all exist and are wired into
  `lib.rs`, directly contradicting the roadmap's claim that Phase 3 (ECS) had "just started" and Phases
  4/6 were pure future work; (2) read `changelog.md` — its `0.1.0 - 2024-08-08` entry already claims
  complete coordinate systems, ECS, FOV, flow fields, examples, and a benchmark suite; (3) read
  `docs/pitfall/readme.md` and `docs/definition/readme.md` (this crate's authoritative `docs/` doc
  definitions per this repo's DEV-repo convention) — found 4 documented pitfalls, two of which
  (`pitfall/001` Flow Field stub, `pitfall/002` ECS movement no-op) mean the changelog's "done" claim for
  those two areas is itself overstated; cross-verified both directly against source
  (`grep -n "stub" src/flowfield.rs` → 10+ "Simplified stub implementation" hits;
  `grep -n "fn request_movement" -A5 src/ecs/world.rs` → target parameter is prefixed `_` and discarded)
  rather than trusting the pitfall docs alone; (4) `grep`-searched `src/` for `wave.function|wfc|tileset|
  noise` — zero hits, confirming Phase 5 (Procedural Generation) is genuinely unstarted, unlike Phases 3/4
  which are partially done; (5) `ls examples/tiles_tools/` — 12 example crates exist, matching
  `readme.md`'s own example table, confirming Phase 6's example deliverables are done beyond the
  original 3-example scope; (6) confirmed `benches/` holds 2 criterion-based benchmark suites, closing
  Phase 6's "Benchmarking suite" deliverable; (7) launched `cargo nextest run -p tiles_tools
  --all-features` via `longrun` (package-scoped per `longrun.rulebook.md § Breadth Selection` — this is a
  single-crate ground-truth check, not a final workspace gate) — 237/237 passed, replacing the stale
  unstamped "159 passing tests" claim with a reproducible, timestamped command instead of a new number
  that will just as surely go stale. Resolution: rewrote the file with one unambiguous status per phase
  (Phases 1/2/6 complete, Phase 3 complete except `pitfall/002`, Phase 4 complete for FOV but Flow Fields
  is `pitfall/001`-stubbed and region-analysis/flood-fill was never started, Phase 5 fully unstarted); kept
  the Milestone 01-11 numbering and phase structure per the task's explicit "fix is accuracy, not removal"
  instruction, condensing Milestone 08's now-irrelevant hour-by-hour task template into the same concise
  historical-summary style already used for Milestones 01-07 in the original file; deduplicated two
  literal repeats of "### Risk Mitigation" and "### Success Metrics" down to one each; fixed Milestone 01's
  dead `docs/ecs_decision.md` deliverable link to the real, current
  `docs/architectural_evaluation/001_ecs_library_selection.md` path (found via the definition/readme.md
  master table); fixed "Files to Understand First" citing `src/coordinates/mod.rs`, which does not exist,
  to the real `src/coordinates.rs`; added a `Known Gaps` section linking (not duplicating) all 4
  `docs/pitfall/` entries; added a `Revision History` section documenting this rewrite and why, so a future
  reader can tell this version apart from a re-drifted one. Dual-Role Self-Check adversarial pass caught
  one omission — `src/layout.rs` (180 LOC, `RectangularGrid` over hex offset coordinates, wired into
  `lib.rs`, no `docs/` instance) was missing from the initial Implementation Status Dashboard — added
  before finalizing.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 Dual-Role Self-Check (D1-D8; this is a
  documentation-accuracy/rewrite task, not a code-defect fix, so B1-B7 Bug-Fixing Quality Requirements do
  not apply — consistent with tasks 023/034/012/019 this session). Confirming pass re-read the full
  rewritten `roadmap.md` and cross-checked every status claim against the investigation trail above.
  Adversarial pass grepped the finished file for leftover stale phrases (`0/4 milestones`, `Milestone 08
  starting`, `🎉`, `COMPLETED!`, `3 weeks from today/now`, `Blocked \|`, `Active \|`) — all matches were
  confined to the intentional Revision-note/Revision-History quotations of the old text, none asserted as
  current state; checked every internal `#anchor` link resolves to a real heading (5/5); checked every
  `docs/...` link resolves to a file that exists (verified against the `ls docs/*/`  sweep and
  `definition/readme.md`'s master table, 15/15); checked for duplicate headings (`grep | sort | uniq -d` →
  none) — this file's own prior version had two literal duplicate headings (`### Risk Mitigation`,
  `### Success Metrics`), so this check was not perfunctory. Adversarial pass caught one real omission
  (`src/layout.rs` missing from the dashboard) — fixed before finalizing. All 8 dimensions 🟢, see
  Verification Record below. State set to ✅ Completed; moved `draft/` → `completed/`.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Draft-stage Goal-only format; Goal names the exact file, the exact contradiction (Phase 2 "COMPLETED" vs "0/4 milestones"), and the required approach (ground-truth rewrite, not deletion) | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (self-contradictory status actively misleads contributors), Observable (the two contradicting sections either exist or don't; test count is independently re-runnable), Scoped (1 file), Testable (grep for the specific contradictory phrases; re-run `cargo nextest`) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skip → a contributor reads "0/4 milestones, Milestone 08 starting today" and wastes time reimplementing Square/Triangular/Isometric/Conversion coordinates that have existed since 0.1.0; investigation also surfaced that the roadmap undersold real completeness (ECS, FOV, examples, benchmarks all already done) as much as it oversold Phase 2 — both directions needed correcting, neither was speculative | — |
| D4 | Implementation Readiness | — | 🟢 | Ground truth gathered before any edit: full `src/**/*.rs` listing, `changelog.md`, `docs/pitfall/` + `docs/definition/readme.md`, direct source cross-checks (`grep stub`, `request_movement` body), a live `cargo nextest` run, example/bench directory listings | — |
| D5 | Execution Scope | — | 🟢 | Single file edited (`module/helper/tiles_tools/roadmap.md`); all cited `docs/` paths and example names verified to exist before being linked, not asserted from memory | — |
| D6 | Crate Scope Unity | — | 🟢 | Edit confined entirely to `module/helper/tiles_tools/roadmap.md` | — |
| D7 | Crate Locality | — | 🟢 | Fix applied directly in the owning crate's own roadmap file, no aggregator or workspace-level doc touched | — |
| D8 | Crate Single Responsibility | — | 🟢 | No responsibility change — correcting a status document's accuracy, not altering what the crate does | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 8 dimensions clean on both passes, zero Blocking Findings. One
Non-Blocking finding (missing `layout.rs` dashboard row) was caught by the adversarial pass and fixed
in place before this table was written, per `governance/maav.rulebook.md § MAAV : Severity-Tiered
Convergence` (fixed findings don't need to survive as open issues in the table). D1–D8 are the Readiness
Verification Gate dimensions, reused at completion per this session's established precedent for
decision/hygiene/doc-rewrite tasks (matching tasks 012/019/023/034) — not a defect fix, so Bug-Fixing
Task Quality Requirements (B1–B7) do not apply.
