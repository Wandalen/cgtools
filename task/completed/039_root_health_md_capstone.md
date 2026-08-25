# Populate root health.md as a living workspace health dashboard (capstone — do last)

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

Root `health.md` is confirmed (direct read this session) to be a literal empty stub — just the title
`# cgtools health`, no body. P-capstone (do last, after the rest of this triage plan's tasks land):
populate it as a living per-crate health dashboard (build status, test coverage, doc-entity adoption,
open marker count, known deprecated/delete-candidate crates) that summarizes the outcome of tasks
008-038. Deliberately sequenced last — writing this before the other fixes land would just document a
known-broken state and need immediate revision; **do not start this until at least the P0 (task 008) and
P1 (soundness bug) tier tasks have landed**, so the dashboard reflects real post-fix state rather than
being stale on arrival.

## In Scope

- `health.md` (workspace root): populated from an empty stub into a living dashboard — dated build-status snapshot header, regeneration-commands table, a 30-row per-`module/`-crate table (tests, docs/ adoption, marker count, allow count, notes), a known-issues section, and an open-streams section
- This task file's own Verification Record and the task index entry

## Out of Scope

- The `examples/` tree — deliberately summarized rather than tabulated per-crate (~50 demo crates, no `tests/` requirement)
- Fixing the underlying issues `health.md` reports (e.g. the `primitive_generation`/`text_rendering`/`morph_targets` build failures found while verifying) — this task only reports state
- Duplicating the task backlog inside `health.md` — the open-streams section links to draft ranges instead of copying content; `task/readme.md` remains the single live tracker

## Verification

### Checklist

- [x] C1 — Does root `health.md` still exist, populated (not the empty stub this task started from)? `wc -l health.md` → `87` lines (was `2`, see M1).
- [x] C2 — Does it retain the 5 structural elements this task's History claims to have written (dated snapshot header with build status, regeneration-commands table, per-crate table, known-issues section, open-streams section)? Direct read confirms all 5 sections present, in that order.
- [x] C3 — Is the "Task system: X completed · Y draft · Z cancelled" line — the specific line this task's own `GATE_CHECKED_AND_COMPLETED` History entry says it corrected for accuracy — still internally well-formed and currently accurate against real directory counts (not just structurally present)? Current text reads "59 completed · 3 draft · 6 cancelled"; `find task/completed -maxdepth 1 -type f | wc -l` / `task/draft` / `task/cancelled` → `59` / `3` / `6` — exact match. (Note: these are *not* the `39 completed · 22 draft` figures this task's own History records fixing to at completion time — the counts have since moved further, consistent with `health.md`'s designed living-document nature; see AF1.)
- [x] C4 — Does the per-crate table's `docs/` column still match the real 8-crate adoption list independently re-derived for task 037's own Verification (C2 there)? Cross-checked — identical set.

### Measurements

- [x] M1 — `health.md` line count: `87` (was: `2` — literally just the title line `# cgtools health` plus a trailing blank line, confirmed via `git show 575935d5:health.md`, the commit that first created the file).

### Invariants

- [x] I1 — DRIFT (critical, currently-live): the exact command this task's own Goal/History cites as its build-status evidence, `cargo check --workspace --all-features` → **currently fails**, exit 101, in 47s (was claimed: exit 0, 57s). Re-run with `--keep-going` to enumerate every failure rather than stopping at the first: 3 crates currently fail to compile under this invocation —
  1. `primitive_generation` (lib): 2× `E0639` in `ufo.rs:83,368` — the same `mingl::BoundingBox`-`#[non_exhaustive]` regression documented in full in task 036's own Verification (§ I4 there).
  2. `examples/minwebgl/text_rendering` (bin): 5× `E0639` in `text.rs` (lines 337, 967, 72, 384, 690) — same root cause, a different, already-known, pre-existing issue (explicitly out of scope for this batch's bug-007 file per this session's own task brief, confirmed via zero local `git diff` against that file).
  3. `examples/minwebgl/morph_targets` (bin): 1× `E0308` mismatched-types in `main.rs:130` — a third, unrelated pre-existing issue.
  Also confirmed this is not an `--all-features`-only problem: `cargo check --workspace` (default features) fails identically, because several example crates' own feature requests unify `primitive_generation`'s `font-processing` feature into any workspace-wide build (see task 036's I4 for the exact crate list).
- [x] I2 — Task-system count re-derivation: `find task/completed -maxdepth 1 -type f | wc -l` → `59`; `find task/draft -maxdepth 1 -type f | wc -l` → `3`; `find task/cancelled -maxdepth 1 -type f | wc -l` → `6` — matches `health.md`'s current text exactly (see C3).
- [x] I3 — Known-issues section spot-check, all 4 named items re-verified against current state: (a) `glslangValidator` absence — `command -v glslangValidator` → exit 1, not found, still accurate; (b) 5-crate stale `Wandalen/cg_tools` repository URL — `grep -rn "Wandalen/cg_tools" module/*/*/Cargo.toml` → `0` hits, all 5 named crates now correctly show `https://github.com/Wandalen/cgtools` — **stale, since fixed**; (c) `browser_log` duplicate `licence`+`license` files — `ls module/helper/browser_log/ | grep -i licen` → only `license` exists — **stale, since fixed**; (d) lint-inheritance stragglers (`mdmath_core`, `ndarray_cg`, `embroidery_tools`) — all 3 now carry `[lints]` `workspace = true` — **stale, since fixed**.

### Anti-faking checks

- [x] AF1 — Guards against trusting any single `health.md` line at face value instead of its own cited regeneration command: I1-I3 are exactly those regeneration commands re-run today, and the result is mixed — C3/I2 (task counts) and C4 (docs/ column) are still accurate, while I1 (build status) and 3 of 4 I3 items (known issues) are now stale — proof that "the file exists and looks complete" (C1-C2) is not evidence its claims are current; the file's own design (a regeneration command per column) is precisely the mechanism a future reader must re-run rather than trust.
- [x] AF2 — Guards against silently blaming this task's own work for I1's build failure: I1's root causes (`ufo.rs`'s `BoundingBox` construction, `text_rendering`, `morph_targets`) are all outside anything `039` itself edited (a single markdown file) — confirmed by this task's own `Execution Scope` gate (`D5: One root file populated + this record + index`) and by `git status`/`git log` showing no relationship between `health.md` and any of the 3 failing crates' source files.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, capstone tier (do
  last), Fix-in-place bucket (root file, not crate-scoped).
- **[2026-08-10]** `IMPLEMENTED` — Sequencing precondition satisfied: every task in the 008-038
  triage plan is closed (38 completed at pickup), so the dashboard documents post-fix state, not a
  known-broken one. Populated root `health.md` with: (1) a dated snapshot header carrying real,
  fresh workspace build evidence — `cargo check --workspace --all-features` run this session, exit
  0 in 57s across all module/ + examples/ crates; (2) a regeneration-commands table so every column
  is re-derivable rather than trusted (each metric names the exact command that produced it); (3) a
  30-row per-crate table for module/ (tests files/fns, inline tests, docs/ adoption, marker count,
  allow count, notes linking each crate's open work to its draft task) built from this session's
  own censuses — every number cross-checked: marker column sums to 57, exactly the post-resolution
  census (75 total − 13 examples − 5 doc-quotes); docs/ column matches task 037's 8-crate adoption
  list; allow counts re-swept fresh this session; (4) a known-issues section with per-item
  verification commands (glslangValidator absence + install command, 5 stale-URL blank crates,
  browser_log licence/license duplicate, lint-inheritance stragglers); (5) an open-streams section
  pointing at draft ranges 058/059-065/066-077 without duplicating the backlog (task/readme.md
  stays the single live tracker). Examples tree deliberately not tabulated per-crate: ~50 demo
  crates carry no tests/ requirement; their marker triage is draft 065.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) the drafted task-system line read "38 completed · 22 draft" — false at
  snapshot time (039 itself was still the 23rd draft) and false after close (39 completed); fixed
  to the post-close truth (39 · 22) since the snapshot ships in the same change that closes 039;
  (2) the dashboard's design was checked against the staleness hazard the Goal itself warns about —
  every column got a regeneration command and the backlog is linked rather than copied, so drift
  degrades to "re-run the command" instead of silent wrongness.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Capstone precondition verified before starting: 008-038 all closed | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | All five Goal-named dimensions present: build, coverage, docs adoption, markers, delete-candidates | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | Backlog linked, not duplicated; examples tree summarized not tabulated | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | One root file populated + this record + index | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Workspace-root administrative artifact by design | — |
| D7 | Crate Locality | 🟢 | 🟢 | N/A — no crate artifact | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | health.md summarizes state only; tracker remains task/readme.md | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Every metric and known issue carries its verification command (house recipe rule) | — |
| B2 | Test-First | 🟢 | 🟢 | Build column is fresh empirical evidence: workspace check exit 0, 57s, this session | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Known-issues section lists real, verified defects with reproduction commands | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Empty stub replaced with content — no placeholder padding | — |
| B5 | Fix Verification | 🟡 | 🟢 | Task-state line was wrong at write time (38·22 vs actual 38·23/39·22) | Corrected to post-close truth; marker column cross-summed to census (57 = 75−13−5) |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Session censuses now live in a permanent, regenerable root artifact | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |
