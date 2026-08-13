# Refresh health.md: regenerate per-crate table, known issues, and open work streams

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📦 (Executed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-13
- **blocked_by:** null
- **priority:** 2
- **executing_at:** 2026-08-13
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

`health.md`'s own header states its contract: "Every column is a snapshot with its regeneration
command — re-run the command to refresh a number instead of trusting the table." The file was last
regenerated 2026-08-10; three days and ~15 completed tasks later (058, 065, 083-097, 099-106, plus
the shader-chunk restructure) its numbers, Known Issues, and Open Work Streams sections had drifted:
zero crates actually carry inline `#[test]` functions any more (the test-coverage stream, 066-078,
finished relocating every one of them, but the table still showed 7 crates with nonzero inline
counts), 3 shader/* crates plus the CLI rename were entirely absent from the per-crate table, 2 of
3 "Known issues" were already fixed (stale `Wandalen/cg_tools` URL, browser_log licence/license
duplication), the "Task system:" summary line and both "Open work streams" entries (058, 065) were
stale (both now ✅ Completed), and several crates' `#[allow]` counts read near-zero not because
suppressions were removed but because they'd migrated to `#[expect(...)]`, which the documented
regeneration command doesn't match. Re-run every documented regeneration command against current
repo state and rewrite the file to match.

## In Scope

- `health.md`: Snapshot date, Workspace build line, Task system summary line
- `health.md`: full per-crate table (33 `module/*/*/` crates) — Tests (files/fns), Inline tests,
  docs/, Markers, Allows columns, re-measured via the file's own documented commands; add the 3
  missing `shader/*` crate rows
- `health.md`: Known issues section — drop confirmed-fixed items, keep confirmed-still-real ones
- `health.md`: Open work streams section — drop closed streams, add currently-open ones (056, 098
  Drafts; 094-097/105/106 Executed-pending-acceptance; 107 Verified/concurrently-owned)
- `health.md`: one clarifying caveat paragraph documenting the allow→expect migration's effect on
  the Allows column's regeneration command

## Out of Scope

- Changing the file's own structure (column set, section headers, regeneration-command table) —
  this is a data refresh, not a redesign
- Adding a new "Expects" column — the existing Allows-column caveat paragraph documents the gap
  without expanding the table's shape; a real new column is a larger, separate design decision
- Any code, test, or non-`health.md` documentation change — this task touches exactly one file
- Investigating or acting on task 107 (owned by a concurrent actor, per its own fresh uncommitted
  edit found during this task's execution) — noted for visibility only, not touched

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Every numeric cell in the refreshed per-crate table matches a fresh run of its documented
    regeneration command against current repo state
-   All 33 `module/*/*/` crates present as rows (30 previously tabulated + 3 `shader/*`)
-   `cargo check --workspace --all-features` re-run and its exit code/duration recorded in the
    Workspace build line
-   Known issues and Open work streams sections reflect current task states, not 2026-08-10 states
-   Independent verification passes per this project's Readiness Verification Gate (Tier 2
    Dual-Role Self-Check per this repo's MAAV tier cap)
-   Task state updated to 🎯 on gate pass

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|
| `grep -rn '#\[test\]' <crate>/src` for every crate previously showing nonzero inline tests | Inline tests column | 0 for all 33 crates (relocated by the 066-078 test-coverage stream) |
| `find <crate>/tests -name "*.rs"` / test-fn count for every crate | Tests (files/fns) column | Matches a fresh count, not the 2026-08-10 snapshot's numbers |
| `[ -d <crate>/docs ]` for the 3 new `shader/*` crates | docs/ column | `yes` for all 3 (each ships `docs/api/` and/or `docs/algorithm/`) |
| `grep -rn '#!\?\[ *allow('` vs `'#!\?\[ *expect('` for mdmath_core, ndarray_cg | Allows column accuracy | Allow count ≈0, expect count nonzero — confirms the caveat paragraph's claim rather than asserting it blind |
| `command -v glslangValidator` | Known issues: shader tooling | Not found — issue is still real, stays listed |
| `grep -rln "Wandalen/cg_tools"` outside health.md's own issue text and completed task archives | Known issues: stale URL | No live matches — issue is fixed, dropped from list |
| `find module/helper/browser_log -iname "licen*"` | Known issues: licence/license dup | Exactly one file (`license`) — issue is fixed, dropped from list |
| Read `task/readme.md` Tasks Index state column for 058, 065 | Open work streams accuracy | Both ✅ Completed — dropped from the open list |

## Acceptance Criteria

- Snapshot date reads 2026-08-13; Workspace build line reflects a fresh `cargo check` run (exit
  code + duration); Task system line matches `task/readme.md`'s actual state distribution
- Per-crate table has exactly 33 rows, one per `module/*/*/` crate, each numeric cell traceable to
  a fresh command run (not carried over from 2026-08-10)
- Known issues lists only currently-real issues; both confirmed-fixed items are gone
- Open work streams lists only currently-open items; 058 and 065 are gone, replaced by an accurate
  accounting of 056/098 (Draft), 094-097/105/106 (Executed, acceptance-pending), and 107 (Verified,
  concurrently owned)
- No fabricated numbers — every cell backed by an actual command run this session, not inference
  or copy-forward from the stale table

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the
walk after the task reaches 🔎 Accepting.

### Checklist

**Header block**
- [ ] C1 — Does the Snapshot date read 2026-08-13?
- [ ] C2 — Does the Workspace build line show exit 0 and a duration consistent with the actual
  `cargo check --workspace --all-features` run?
- [ ] C3 — Does the Task system line match `task/readme.md`'s current state distribution?

**Per-crate table**
- [ ] C4 — Are all 33 `module/*/*/` crates present as rows, including the 3 `shader/*` crates?
- [ ] C5 — Does the Inline tests column read 0 for every row?
- [ ] C6 — Do the Tests (files/fns) numbers match a fresh count rather than the 2026-08-10 figures?

**Known issues / Open work streams**
- [ ] C7 — Are the stale-URL and licence/license issues absent from Known issues?
- [ ] C8 — Is `glslangValidator` still listed (still genuinely absent from the machine)?
- [ ] C9 — Are 058 and 065 absent from Open work streams (both ✅ Completed)?
- [ ] C10 — Are 056, 098, 094-097/105/106, and 107 each represented in Open work streams with an
  accurate current state?

### Measurements

- [ ] M1 — `awk '/^\| Crate \| Tests/{p=1;next} p&&/^\|---/{next} p&&/^\|/{c++} p&&!/^\|/{exit} END{print c}' health.md`
  (per-crate table rows, anchored between the `| Crate | Tests...` header and the table's end — the
  naive `grep -c '^| [a-z]'` over-matches the regeneration-commands table's `| docs/ | ...` row,
  reading 34 instead of the true 33) → 33
- [ ] M2 — `grep -rln '#\[test\]' module/*/*/src 2>/dev/null | wc -l` → 0 (confirms C5 against
  ground truth, not just the table's own claim)

### Invariants

- [ ] I1 — `command -v glslangValidator` → not found (confirms the still-real issue wasn't
  accidentally dropped)
- [ ] I2 — `git diff --stat -- health.md` shows only `health.md` changed — no other file touched

### Anti-faking checks

- [ ] AF1 — Spot-check 3 arbitrary numeric cells against a live re-run of their documented command
  — must match exactly, not merely "look plausible"
- [ ] AF2 — `grep -c "2026-08-10" health.md` → 0 (no stale snapshot-date residue left anywhere in
  the file, including inside Notes-column prose)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope is a bounded, single-file data refresh; Out of Scope explicitly excludes redesign and the concurrently-owned task 107 | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated by the file's own regeneration contract going stale; Observable (numbers match fresh commands); Scoped (one file); Testable (Measurements re-run the same greps) | — |
| G3 | Value/YAGNI | — | 🟢 | Real, concrete drift found and fixed (0-inline-tests-everywhere, 3 missing crates, 2 fixed issues still listed, 2 closed streams still listed) — not speculative maintenance | — |
| G4 | Implementation Readiness | — | 🟢 | Work Procedure is direct command re-runs against documented commands; no ambiguity in what "regenerate" means since the file defines its own commands | — |
| G5 | Execution Scope | — | 🟢 | `health.md` resolves inside this repository | — |
| G6 | Crate Scope Unity | — | 🟢 | Single file at repo root, not spread across crate boundaries — `unit_type: workspace` is correct, not a crate-scoped task mis-tagged | — |
| G7 | Crate Locality | — | 🟢 | N/A at workspace scope — health.md is inherently a cross-crate dashboard, not owned by any one crate | — |
| G8 | Crate Single Responsibility | — | 🟢 | Task's sole responsibility is data accuracy of one dashboard file — no second concern bundled in | — |
| **Total** | | — | 🟢 | 0 blocking | — |

Adversarial pass (summary): challenged whether "0 inline tests everywhere" was a measurement bug
rather than genuine current state — resolved by direct spot-checks against `mingl` and `renderer`
(both previously-documented nonzero exceptions) confirming zero `#[test]` matches and zero
`#[cfg(test)]` blocks in `src/`, git log showing repeated "consolidate/modernize test
infrastructure" commits consistent with a completed relocation, not a script defect. Challenged
whether the Allows-column near-zero readings for mdmath_core/ndarray_cg represented a real
suppression removal or a measurement gap — resolved by a direct allow-vs-expect comparison grep
confirming the counts moved to `#[expect(...)]`, not to zero suppressions; documented as a caveat
rather than silently presenting a misleadingly-clean number. No blocking finding surfaced.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | All Delivery Requirements met: re-ran `cargo check --workspace --all-features` via longrun detached launch (exit 0, 108s, log `task/-0017_longrun.log`); re-ran the per-crate metrics loop (find/grep) against all 33 `module/*/*/` crates including the 3 previously-untabulated `shader/*` crates; confirmed genuine zero inline `#[test]`/`#[cfg(test)]` matches workspace-wide (not a script bug — spot-verified directly against mingl and renderer, both previously-documented nonzero exceptions, plus a repo-wide `grep -rln '#[test]' module/*/*/src` returning 0 files); confirmed mdmath_core/ndarray_cg's near-zero Allows readings reflect an allow→expect migration (39 and 10 expects respectively), not suppression removal, and added a caveat paragraph rather than presenting a misleading number; re-verified all 3 Known Issues against current state (glslangValidator still absent — kept; stale `Wandalen/cg_tools` URL and browser_log licence/license duplication both confirmed fixed via direct grep/find — dropped); rewrote Open work streams to drop 058/065 (✅ Completed) and add accurate current entries for 056/098 (📝 Draft), 094-097/105/106 (📦 Executed, acceptance-pending per the Separation-of-Concerns/BUG-197 mechanical block), and 107 (🎯 Verified, filed by a different concurrent actor in this workspace — confirmed via a fresh uncommitted diff to its own file plus `docs/pattern/004_script_as_data.md`, both untouched by this task per Out of Scope). Full file written via three `Edit` calls (header block, per-crate table + explanatory paragraph, Known issues + Open work streams), never `Write`, to keep the operation a diffable patch against the version last read rather than a blind overwrite — relevant given a concurrent actor is independently editing other files in this same `task/` tree during this session. Checklist/Measurements/Invariants/Anti-faking boxes deliberately left unchecked — Verification section states the executor does not self-verify; leaving for an independent verifier per Claim Accept (📦→🔎). |
| 2026-08-13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | SELF_CORRECTION | Post-Gate-Check adversarial recheck (same session) found the Verification Record's 8/8 PASS was based on two undetected defects in this task file itself, both now fixed: (1) `**priority:** 3` in Execution State didn't match the Tasks Index row's Priority=2/Advisability=576 (4×8×9×2=576, not ×3=864) — corrected to `2`, matching the already-registered Index arithmetic. (2) Every "32 crates" claim (In Scope, Delivery Requirements, Test Matrix, Acceptance Criteria, C4, M1, this Journal's own EXEC_COMPLETE text) was wrong — mechanically re-verified via `awk` anchored between the `| Crate \| Tests` header and table end, cross-checked against a live `find module -mindepth 3 -maxdepth 3 -name Cargo.toml` filesystem listing: both agree on exactly 33 rows (30 previously-tabulated, not 29, + 3 shader/*), and the diff between the two listings is empty. M1's own measurement command was independently broken regardless of the count: `grep -c '^\| [a-z]' health.md` returns 34, not 33, because it also matches the unrelated regeneration-commands table's `\| docs/ \| ...` row (lowercase `d`) — replaced with an awk command anchored to the actual table boundaries. health.md itself needed no changes — its table was already correct at 33; only this task file's documentation of it was wrong. |

## History

- **2026-08-13** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: refresh health.md's per-crate table, Known Issues, and Open Work Streams sections, stale since the 2026-08-10 snapshot. Filed retroactively documenting work already performed this session at the user's explicit request ("good. do all that").
